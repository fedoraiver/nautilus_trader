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

use std::{collections::BTreeSet, env, time::Duration};

use chainlink_data_streams_sdk::config::{
    Config as ChainlinkSdkConfig, InsecureSkipVerify, WebSocketHighAvailability,
};
use nautilus_core::string::secret::REDACTED;
use serde::{Deserialize, Serialize};

use crate::common::{
    canonicalize_feed_id,
    consts::{
        DEFAULT_API_BASE_URL, DEFAULT_WS_BASE_URL, ENV_API_BASE_URL, ENV_API_KEY, ENV_API_SECRET,
        ENV_WS_BASE_URL,
    },
};

/// Configuration for the Chainlink data client.
#[derive(Clone, Serialize, Deserialize, bon::Builder)]
#[serde(default, deny_unknown_fields)]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(
        module = "nautilus_trader.core.nautilus_pyo3.chainlink",
        from_py_object
    )
)]
#[cfg_attr(
    feature = "python",
    pyo3_stub_gen::derive::gen_stub_pyclass(module = "nautilus_trader.chainlink")
)]
pub struct ChainlinkDataClientConfig {
    /// Chainlink Data Streams API key.
    pub api_key: Option<String>,
    /// Chainlink Data Streams API secret.
    pub api_secret: Option<String>,
    /// Chainlink REST base URL.
    pub api_base_url: Option<String>,
    /// Chainlink WebSocket base URL.
    pub ws_base_url: Option<String>,
    /// Feed IDs to subscribe immediately on connect.
    #[builder(default)]
    pub feed_ids: Vec<String>,
    /// Adapter-level reconnect delay when the stream must be recreated.
    #[builder(default = 5.0)]
    pub reconnect_delay_secs: f64,
    /// Enables Chainlink SDK WebSocket high availability mode.
    #[builder(default)]
    pub ws_high_availability: bool,
    /// Maximum SDK reconnect attempts per underlying WebSocket connection.
    #[builder(default = 5)]
    pub ws_max_reconnect: usize,
    /// Skip TLS verification for the Chainlink SDK.
    #[builder(default)]
    pub insecure_skip_verify: bool,
}

impl std::fmt::Debug for ChainlinkDataClientConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(stringify!(ChainlinkDataClientConfig))
            .field(
                "api_key",
                &self.api_key.as_ref().map_or("None", |_| REDACTED),
            )
            .field(
                "api_secret",
                &self.api_secret.as_ref().map_or("None", |_| REDACTED),
            )
            .field("api_base_url", &self.api_base_url)
            .field("ws_base_url", &self.ws_base_url)
            .field("feed_ids", &self.feed_ids)
            .field("reconnect_delay_secs", &self.reconnect_delay_secs)
            .field("ws_high_availability", &self.ws_high_availability)
            .field("ws_max_reconnect", &self.ws_max_reconnect)
            .field("insecure_skip_verify", &self.insecure_skip_verify)
            .finish()
    }
}

impl Default for ChainlinkDataClientConfig {
    fn default() -> Self {
        Self::builder().build()
    }
}

#[derive(Clone)]
pub(crate) struct ResolvedChainlinkConfig {
    pub sdk_config: ChainlinkSdkConfig,
    pub reconnect_delay: Duration,
}

impl std::fmt::Debug for ResolvedChainlinkConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(stringify!(ResolvedChainlinkConfig))
            .field("sdk_config", &"ChainlinkSdkConfig(..)")
            .field("reconnect_delay", &self.reconnect_delay)
            .finish()
    }
}

impl ChainlinkDataClientConfig {
    /// Returns the configured feed IDs, validated and canonicalized.
    ///
    /// # Errors
    ///
    /// Returns an error if any configured feed ID is invalid.
    pub(crate) fn validated_feed_ids(&self) -> anyhow::Result<BTreeSet<String>> {
        self.feed_ids
            .iter()
            .map(|id| canonicalize_feed_id(id))
            .collect()
    }

    /// Resolves env-backed credentials and builds the Chainlink SDK config.
    ///
    /// # Errors
    ///
    /// Returns an error if credentials are missing or the SDK config cannot be built.
    pub(crate) fn resolve(&self) -> anyhow::Result<ResolvedChainlinkConfig> {
        if self.reconnect_delay_secs <= 0.0 {
            anyhow::bail!("Chainlink reconnect_delay_secs must be greater than 0");
        }

        let api_key = self
            .api_key
            .clone()
            .or_else(|| env::var(ENV_API_KEY).ok())
            .ok_or_else(|| anyhow::anyhow!("{ENV_API_KEY} is required"))?;
        let api_secret = self
            .api_secret
            .clone()
            .or_else(|| env::var(ENV_API_SECRET).ok())
            .ok_or_else(|| anyhow::anyhow!("{ENV_API_SECRET} is required"))?;
        let api_base_url = self
            .api_base_url
            .clone()
            .or_else(|| env::var(ENV_API_BASE_URL).ok())
            .unwrap_or_else(|| DEFAULT_API_BASE_URL.to_string());
        let ws_base_url = self
            .ws_base_url
            .clone()
            .or_else(|| env::var(ENV_WS_BASE_URL).ok())
            .unwrap_or_else(|| DEFAULT_WS_BASE_URL.to_string());

        let ws_ha = if self.ws_high_availability {
            WebSocketHighAvailability::Enabled
        } else {
            WebSocketHighAvailability::Disabled
        };
        let insecure_skip_verify = if self.insecure_skip_verify {
            InsecureSkipVerify::Enabled
        } else {
            InsecureSkipVerify::Disabled
        };

        let sdk_config = ChainlinkSdkConfig::new(api_key, api_secret, api_base_url, ws_base_url)
            .with_ws_ha(ws_ha)
            .with_ws_max_reconnect(self.ws_max_reconnect)
            .with_insecure_skip_verify(insecure_skip_verify)
            .build()
            .map_err(|err| anyhow::anyhow!("Failed to build Chainlink SDK config: {err}"))?;

        Ok(ResolvedChainlinkConfig {
            sdk_config,
            reconnect_delay: Duration::from_secs_f64(self.reconnect_delay_secs),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_values() {
        let config = ChainlinkDataClientConfig::default();
        assert!(config.api_key.is_none());
        assert!(config.api_secret.is_none());
        assert!(config.api_base_url.is_none());
        assert!(config.ws_base_url.is_none());
        assert!(config.feed_ids.is_empty());
        assert_eq!(config.reconnect_delay_secs, 5.0);
        assert!(!config.ws_high_availability);
        assert_eq!(config.ws_max_reconnect, 5);
        assert!(!config.insecure_skip_verify);
    }
}
