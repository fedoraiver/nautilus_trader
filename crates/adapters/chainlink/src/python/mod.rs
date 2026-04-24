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

//! Python bindings from [PyO3](https://pyo3.rs).

#![expect(
    clippy::missing_errors_doc,
    reason = "errors documented on underlying Rust methods"
)]

pub mod factories;

use nautilus_common::factories::{ClientConfig, DataClientFactory};
use nautilus_core::python::{to_pyruntime_err, to_pyvalue_err};
use nautilus_model::data::ensure_custom_data_json_registered;
use nautilus_system::get_global_pyo3_registry;
use pyo3::prelude::*;

use crate::{
    config::ChainlinkDataClientConfig, factories::ChainlinkDataClientFactory, types::ChainlinkData,
};

#[expect(clippy::needless_pass_by_value)]
fn extract_chainlink_data_factory(
    py: Python<'_>,
    factory: Py<PyAny>,
) -> PyResult<Box<dyn DataClientFactory>> {
    match factory.extract::<ChainlinkDataClientFactory>(py) {
        Ok(factory) => Ok(Box::new(factory)),
        Err(e) => Err(to_pyvalue_err(format!(
            "Failed to extract ChainlinkDataClientFactory: {e}"
        ))),
    }
}

#[expect(clippy::needless_pass_by_value)]
fn extract_chainlink_data_config(
    py: Python<'_>,
    config: Py<PyAny>,
) -> PyResult<Box<dyn ClientConfig>> {
    match config.extract::<ChainlinkDataClientConfig>(py) {
        Ok(config) => Ok(Box::new(config)),
        Err(e) => Err(to_pyvalue_err(format!(
            "Failed to extract ChainlinkDataClientConfig: {e}"
        ))),
    }
}

/// Chainlink Python module.
#[pymodule]
pub fn chainlink(_: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    ensure_custom_data_json_registered::<ChainlinkData>().map_err(to_pyruntime_err)?;

    m.add_class::<ChainlinkData>()?;
    m.add_class::<ChainlinkDataClientConfig>()?;
    m.add_class::<ChainlinkDataClientFactory>()?;

    let registry = get_global_pyo3_registry();

    registry
        .register_factory_extractor("CHAINLINK".to_string(), extract_chainlink_data_factory)
        .map_err(|e| {
            to_pyruntime_err(format!(
                "Failed to register Chainlink data factory extractor: {e}"
            ))
        })?;

    registry
        .register_config_extractor(
            "ChainlinkDataClientConfig".to_string(),
            extract_chainlink_data_config,
        )
        .map_err(|e| {
            to_pyruntime_err(format!(
                "Failed to register Chainlink data config extractor: {e}"
            ))
        })?;

    Ok(())
}
