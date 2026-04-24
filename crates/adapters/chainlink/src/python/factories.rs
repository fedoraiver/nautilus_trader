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

use pyo3::prelude::*;

use crate::{
    common::consts::CHAINLINK, config::ChainlinkDataClientConfig,
    factories::ChainlinkDataClientFactory,
};

#[pymethods]
#[pyo3_stub_gen::derive::gen_stub_pymethods]
impl ChainlinkDataClientConfig {
    /// Configuration for Chainlink data clients used with `LiveNodeBuilder`.
    #[new]
    #[pyo3(signature = (
        api_key = None,
        api_secret = None,
        api_base_url = None,
        ws_base_url = None,
        feed_ids = None,
        reconnect_delay_secs = 5.0,
        ws_high_availability = false,
        ws_max_reconnect = 5,
        insecure_skip_verify = false
    ))]
    #[expect(clippy::too_many_arguments)]
    fn py_new(
        api_key: Option<String>,
        api_secret: Option<String>,
        api_base_url: Option<String>,
        ws_base_url: Option<String>,
        feed_ids: Option<Vec<String>>,
        reconnect_delay_secs: f64,
        ws_high_availability: bool,
        ws_max_reconnect: usize,
        insecure_skip_verify: bool,
    ) -> Self {
        Self {
            api_key,
            api_secret,
            api_base_url,
            ws_base_url,
            feed_ids: feed_ids.unwrap_or_default(),
            reconnect_delay_secs,
            ws_high_availability,
            ws_max_reconnect,
            insecure_skip_verify,
        }
    }

    fn __repr__(&self) -> String {
        format!("{self:?}")
    }
}

#[pymethods]
#[pyo3_stub_gen::derive::gen_stub_pymethods]
impl ChainlinkDataClientFactory {
    /// Factory for creating Chainlink data clients.
    #[new]
    fn py_new() -> Self {
        Self
    }

    #[pyo3(name = "name")]
    fn py_name(&self) -> &str {
        CHAINLINK
    }
}
