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

//! Factory functions for creating Chainlink data clients.

use std::{any::Any, cell::RefCell, rc::Rc};

use nautilus_common::{
    cache::CacheView,
    clients::DataClient,
    clock::Clock,
    factories::{ClientConfig, DataClientFactory},
};
use nautilus_model::identifiers::ClientId;

use crate::{
    common::consts::CHAINLINK, config::ChainlinkDataClientConfig, data::ChainlinkDataClient,
};

impl ClientConfig for ChainlinkDataClientConfig {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Factory for creating Chainlink data clients.
#[derive(Debug, Clone)]
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
pub struct ChainlinkDataClientFactory;

impl ChainlinkDataClientFactory {
    /// Creates a new `ChainlinkDataClientFactory`.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Default for ChainlinkDataClientFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl DataClientFactory for ChainlinkDataClientFactory {
    fn create(
        &self,
        name: &str,
        config: &dyn ClientConfig,
        _cache: CacheView,
        _clock: Rc<RefCell<dyn Clock>>,
    ) -> anyhow::Result<Box<dyn DataClient>> {
        let config = config
            .as_any()
            .downcast_ref::<ChainlinkDataClientConfig>()
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Invalid config type for ChainlinkDataClientFactory. Expected ChainlinkDataClientConfig, was {config:?}"
                )
            })?
            .clone();

        let client_id = ClientId::from(name);
        let client = ChainlinkDataClient::new(client_id, config)?;
        Ok(Box::new(client))
    }

    fn name(&self) -> &'static str {
        CHAINLINK
    }

    fn config_type(&self) -> &'static str {
        "ChainlinkDataClientConfig"
    }
}
