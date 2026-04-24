// -------------------------------------------------------------------------------------------------
//  Copyright (C) 2015-2026 Nautech Systems Pty Ltd. All rights reserved.
//  https://nautechsystems.io
//
//  Licensed under the GNU Lesser General Public License Version 3.0 (the "License");
//  You may not use this file except in compliance with the License.
//  You may obtain a copy of the License at https://www.gnu.org/licenses/lgpl-3.0.en.html
//
//  Unless required by applicable law or agreed to in writing, software
//  distributed under the License is distributed on an "AS IS" BASIS,
//  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//  See the License for the specific language governing permissions and
//  limitations under the License.
// -------------------------------------------------------------------------------------------------

//! Chainlink live data client for feeding Data Streams reports into Nautilus.

use std::{
    collections::BTreeSet,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use async_trait::async_trait;
use chainlink_data_streams_report::feed_id::ID;
use chainlink_data_streams_sdk::stream::Stream;
use nautilus_common::{
    clients::DataClient,
    live::{runner::get_data_event_sender, runtime::get_runtime},
    messages::{
        DataEvent,
        data::{SubscribeCustomData, UnsubscribeCustomData},
    },
};
use nautilus_core::{Params, time::get_atomic_clock_realtime};
use nautilus_model::{
    data::{CustomData, Data, register_custom_data_json},
    identifiers::{ClientId, Venue},
};
use serde_json::Value;
use tokio::{
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
    task::JoinHandle,
};

use crate::{
    common::{
        canonicalize_feed_id,
        consts::{CHAINLINK_DATA_TYPE, FEED_ID_KEY, FEED_IDS_KEY},
    },
    config::{ChainlinkDataClientConfig, ResolvedChainlinkConfig},
    types::ChainlinkData,
};

#[derive(Debug)]
enum WorkerCommand {
    SetFeedIds(BTreeSet<String>),
    Shutdown,
}

/// Chainlink live data client.
#[derive(Debug)]
pub struct ChainlinkDataClient {
    client_id: ClientId,
    config: ChainlinkDataClientConfig,
    configured_feed_ids: BTreeSet<String>,
    subscribed_feed_ids: BTreeSet<String>,
    is_connected: Arc<AtomicBool>,
    command_tx: Option<UnboundedSender<WorkerCommand>>,
    task_handle: Option<JoinHandle<()>>,
    data_sender: UnboundedSender<DataEvent>,
}

impl ChainlinkDataClient {
    /// Creates a new `ChainlinkDataClient`.
    ///
    /// # Errors
    ///
    /// Returns an error if the config contains invalid feed IDs or the custom data
    /// type cannot be registered.
    pub fn new(client_id: ClientId, config: ChainlinkDataClientConfig) -> anyhow::Result<Self> {
        let configured_feed_ids = config.validated_feed_ids()?;
        register_custom_data_json::<ChainlinkData>()?;

        Ok(Self {
            client_id,
            config,
            configured_feed_ids,
            subscribed_feed_ids: BTreeSet::new(),
            is_connected: Arc::new(AtomicBool::new(false)),
            command_tx: None,
            task_handle: None,
            data_sender: get_data_event_sender(),
        })
    }

    fn effective_feed_ids(&self) -> BTreeSet<String> {
        self.configured_feed_ids
            .union(&self.subscribed_feed_ids)
            .cloned()
            .collect()
    }

    fn cleanup_finished_worker(&mut self) {
        if self
            .task_handle
            .as_ref()
            .is_some_and(tokio::task::JoinHandle::is_finished)
        {
            self.task_handle.take();
            self.command_tx.take();
            self.is_connected.store(false, Ordering::Release);
        }
    }

    fn send_feed_update(&mut self) -> anyhow::Result<()> {
        self.cleanup_finished_worker();

        if let Some(tx) = &self.command_tx {
            let feed_ids = self.effective_feed_ids();
            tx.send(WorkerCommand::SetFeedIds(feed_ids))
                .map_err(|err| anyhow::anyhow!("Failed to update Chainlink subscriptions: {err}"))
        } else {
            Ok(())
        }
    }
}

#[async_trait(?Send)]
impl DataClient for ChainlinkDataClient {
    fn client_id(&self) -> ClientId {
        self.client_id
    }

    fn venue(&self) -> Option<Venue> {
        None
    }

    fn start(&mut self) -> anyhow::Result<()> {
        log::info!("Starting {}", self.client_id);
        Ok(())
    }

    fn stop(&mut self) -> anyhow::Result<()> {
        log::info!("Stopping {}", self.client_id);
        if let Some(tx) = self.command_tx.take() {
            let _ = tx.send(WorkerCommand::Shutdown);
        }
        if let Some(handle) = self.task_handle.take() {
            handle.abort();
        }
        self.is_connected.store(false, Ordering::Release);
        Ok(())
    }

    fn reset(&mut self) -> anyhow::Result<()> {
        self.stop()?;
        self.subscribed_feed_ids.clear();
        Ok(())
    }

    fn dispose(&mut self) -> anyhow::Result<()> {
        self.stop()
    }

    fn is_connected(&self) -> bool {
        self.is_connected.load(Ordering::Acquire)
    }

    fn is_disconnected(&self) -> bool {
        !self.is_connected()
    }

    fn subscribe(&mut self, cmd: SubscribeCustomData) -> anyhow::Result<()> {
        let feed_ids = extract_feed_ids(&cmd.data_type, cmd.params.as_ref())?;
        self.subscribed_feed_ids.extend(feed_ids.clone());
        log::info!("Subscribed Chainlink feeds: {}", join_feed_ids(&feed_ids));
        self.send_feed_update()
    }

    fn unsubscribe(&mut self, cmd: &UnsubscribeCustomData) -> anyhow::Result<()> {
        let feed_ids = extract_feed_ids(&cmd.data_type, cmd.params.as_ref())?;
        for feed_id in &feed_ids {
            self.subscribed_feed_ids.remove(feed_id);
        }
        log::info!("Unsubscribed Chainlink feeds: {}", join_feed_ids(&feed_ids));
        self.send_feed_update()
    }

    async fn connect(&mut self) -> anyhow::Result<()> {
        self.cleanup_finished_worker();

        if self.command_tx.is_some() {
            self.send_feed_update()?;
            return Ok(());
        }

        let resolved = self.config.resolve()?;
        let initial_feed_ids = self.effective_feed_ids();
        let (command_tx, command_rx) = tokio::sync::mpsc::unbounded_channel();
        let data_sender = self.data_sender.clone();
        let is_connected = Arc::clone(&self.is_connected);

        self.task_handle = Some(get_runtime().spawn(async move {
            run_worker(
                resolved,
                initial_feed_ids,
                command_rx,
                data_sender,
                is_connected,
            )
            .await;
        }));
        self.command_tx = Some(command_tx);

        Ok(())
    }

    async fn disconnect(&mut self) -> anyhow::Result<()> {
        if let Some(tx) = self.command_tx.take() {
            let _ = tx.send(WorkerCommand::Shutdown);
        }

        if let Some(handle) = self.task_handle.take() {
            let _ = handle.await;
        }

        self.is_connected.store(false, Ordering::Release);
        Ok(())
    }
}

async fn run_worker(
    resolved: ResolvedChainlinkConfig,
    mut feed_ids: BTreeSet<String>,
    mut command_rx: UnboundedReceiver<WorkerCommand>,
    data_sender: UnboundedSender<DataEvent>,
    is_connected: Arc<AtomicBool>,
) {
    loop {
        if !drain_commands(&mut command_rx, &mut feed_ids, &is_connected) {
            return;
        }

        if feed_ids.is_empty() {
            match command_rx.recv().await {
                Some(WorkerCommand::SetFeedIds(next_feed_ids)) => {
                    feed_ids = next_feed_ids;
                    continue;
                }
                Some(WorkerCommand::Shutdown) | None => {
                    is_connected.store(false, Ordering::Release);
                    return;
                }
            }
        }

        let parsed_feed_ids = match feed_ids
            .iter()
            .map(|feed_id| {
                ID::from_hex_str(feed_id).map_err(|err| {
                    anyhow::anyhow!(
                        "Invalid Chainlink feed ID `{feed_id}` reached worker loop: {err}"
                    )
                })
            })
            .collect::<anyhow::Result<Vec<_>>>()
        {
            Ok(value) => value,
            Err(err) => {
                log::error!("{err}");
                is_connected.store(false, Ordering::Release);
                if !wait_for_retry_or_command(
                    resolved.reconnect_delay,
                    &mut command_rx,
                    &mut feed_ids,
                    &is_connected,
                )
                .await
                {
                    return;
                }
                continue;
            }
        };

        let mut stream = match Stream::new(&resolved.sdk_config, parsed_feed_ids).await {
            Ok(stream) => stream,
            Err(err) => {
                log::warn!("Failed to create Chainlink stream: {err}");
                is_connected.store(false, Ordering::Release);
                if !wait_for_retry_or_command(
                    resolved.reconnect_delay,
                    &mut command_rx,
                    &mut feed_ids,
                    &is_connected,
                )
                .await
                {
                    return;
                }
                continue;
            }
        };

        if let Err(err) = stream.listen().await {
            log::warn!("Failed to start Chainlink stream listener: {err}");
            is_connected.store(false, Ordering::Release);
            if !wait_for_retry_or_command(
                resolved.reconnect_delay,
                &mut command_rx,
                &mut feed_ids,
                &is_connected,
            )
            .await
            {
                return;
            }
            continue;
        }

        is_connected.store(true, Ordering::Release);
        let mut should_retry = false;

        loop {
            tokio::select! {
                biased;

                command = command_rx.recv() => {
                    match command {
                        Some(WorkerCommand::SetFeedIds(next_feed_ids)) => {
                            if next_feed_ids != feed_ids {
                                feed_ids = next_feed_ids;
                                let _ = stream.close().await;
                                break;
                            }
                        }
                        Some(WorkerCommand::Shutdown) | None => {
                            let _ = stream.close().await;
                            is_connected.store(false, Ordering::Release);
                            return;
                        }
                    }
                }
                result = stream.read() => {
                    match result {
                        Ok(report) => {
                            let ts_init = get_atomic_clock_realtime().get_time_ns();
                            match ChainlinkData::from_ws_report(&report, ts_init) {
                                Ok(data) => {
                                    let feed_id = data.feed_id.clone();
                                    let custom = CustomData::new(
                                        Arc::new(data),
                                        ChainlinkData::data_type(&feed_id),
                                    );
                                    if data_sender.send(DataEvent::Data(Data::Custom(custom))).is_err() {
                                        is_connected.store(false, Ordering::Release);
                                        return;
                                    }
                                }
                                Err(err) => {
                                    log::error!("Failed to decode Chainlink report: {err}");
                                }
                            }
                        }
                        Err(err) => {
                            log::warn!("Chainlink stream read failed: {err}");
                            should_retry = true;
                            break;
                        }
                    }
                }
            }
        }

        is_connected.store(false, Ordering::Release);

        if should_retry
            && !wait_for_retry_or_command(
                resolved.reconnect_delay,
                &mut command_rx,
                &mut feed_ids,
                &is_connected,
            )
            .await
        {
            return;
        }
    }
}

fn drain_commands(
    command_rx: &mut UnboundedReceiver<WorkerCommand>,
    feed_ids: &mut BTreeSet<String>,
    is_connected: &AtomicBool,
) -> bool {
    while let Ok(command) = command_rx.try_recv() {
        match command {
            WorkerCommand::SetFeedIds(next_feed_ids) => *feed_ids = next_feed_ids,
            WorkerCommand::Shutdown => {
                is_connected.store(false, Ordering::Release);
                return false;
            }
        }
    }

    true
}

async fn wait_for_retry_or_command(
    delay: std::time::Duration,
    command_rx: &mut UnboundedReceiver<WorkerCommand>,
    feed_ids: &mut BTreeSet<String>,
    is_connected: &AtomicBool,
) -> bool {
    tokio::select! {
        command = command_rx.recv() => {
            match command {
                Some(WorkerCommand::SetFeedIds(next_feed_ids)) => {
                    *feed_ids = next_feed_ids;
                    true
                }
                Some(WorkerCommand::Shutdown) | None => {
                    is_connected.store(false, Ordering::Release);
                    false
                }
            }
        }
        () = tokio::time::sleep(delay) => true,
    }
}

fn extract_feed_ids(
    data_type: &nautilus_model::data::DataType,
    params: Option<&Params>,
) -> anyhow::Result<BTreeSet<String>> {
    if data_type.type_name() != CHAINLINK_DATA_TYPE {
        anyhow::bail!(
            "ChainlinkDataClient only supports `{CHAINLINK_DATA_TYPE}` subscriptions, got `{}`",
            data_type.type_name()
        );
    }

    let mut feed_ids = BTreeSet::new();

    if let Some(metadata) = data_type.metadata() {
        collect_feed_ids(metadata, &mut feed_ids)?;
    }

    if let Some(params) = params {
        collect_feed_ids(params, &mut feed_ids)?;
    }

    if feed_ids.is_empty() {
        anyhow::bail!(
            "Chainlink subscriptions require `{FEED_ID_KEY}` metadata or params on the data type"
        );
    }

    Ok(feed_ids)
}

fn collect_feed_ids(source: &Params, target: &mut BTreeSet<String>) -> anyhow::Result<()> {
    if let Some(feed_id) = source.get_str(FEED_ID_KEY) {
        target.insert(canonicalize_feed_id(feed_id)?);
    }

    if let Some(value) = source.get(FEED_IDS_KEY) {
        match value {
            Value::String(feed_id) => {
                target.insert(canonicalize_feed_id(feed_id)?);
            }
            Value::Array(feed_ids) => {
                for feed_id in feed_ids {
                    let feed_id = feed_id.as_str().ok_or_else(|| {
                        anyhow::anyhow!(
                            "Chainlink `{FEED_IDS_KEY}` entries must be strings, got `{feed_id}`"
                        )
                    })?;
                    target.insert(canonicalize_feed_id(feed_id)?);
                }
            }
            _ => {
                anyhow::bail!("Chainlink `{FEED_IDS_KEY}` must be a string or array of strings");
            }
        }
    }

    Ok(())
}

fn join_feed_ids(feed_ids: &BTreeSet<String>) -> String {
    feed_ids.iter().cloned().collect::<Vec<_>>().join(", ")
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_extract_feed_ids_from_metadata() {
        let mut metadata = Params::new();
        metadata.insert(
            FEED_ID_KEY.to_string(),
            json!("0x00036b4aa7e57ca7b68ae1bf45653f56b656fd3aa335ef7fae696b663f1b8472"),
        );
        let data_type =
            nautilus_model::data::DataType::new(CHAINLINK_DATA_TYPE, Some(metadata), None);

        let feed_ids = extract_feed_ids(&data_type, None).unwrap();
        assert_eq!(feed_ids.len(), 1);
        assert!(
            feed_ids.contains("0x00036b4aa7e57ca7b68ae1bf45653f56b656fd3aa335ef7fae696b663f1b8472")
        );
    }
}
