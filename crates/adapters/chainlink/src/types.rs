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

use std::{collections::HashMap, sync::Arc};

use chainlink_data_streams_report::report::{decode_full_report, v3::ReportDataV3};
use chainlink_data_streams_sdk::stream::WebSocketReport;
use nautilus_core::{Params, UnixNanos};
use nautilus_model::data::{CustomDataTrait, DataType, HasTsInit};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::common::consts::{CHAINLINK_DATA_TYPE, FEED_ID_KEY};

/// Chainlink Data Streams report mapped into Nautilus custom data.
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(
        module = "nautilus_trader.core.nautilus_pyo3.chainlink",
        get_all,
        from_py_object
    )
)]
#[cfg_attr(
    feature = "python",
    pyo3_stub_gen::derive::gen_stub_pyclass(module = "nautilus_trader.chainlink")
)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChainlinkData {
    /// The Chainlink feed ID.
    pub feed_id: String,
    /// Benchmark price as the raw report integer string.
    pub benchmark_price: String,
    /// Bid price as the raw report integer string.
    pub bid: String,
    /// Ask price as the raw report integer string.
    pub ask: String,
    /// Nautilus event timestamp.
    pub ts_event: u64,
    /// Nautilus initialization timestamp.
    pub ts_init: u64,
}

impl ChainlinkData {
    /// Creates the custom-data type used for a specific Chainlink `feed_id`.
    #[must_use]
    pub fn data_type(feed_id: &str) -> DataType {
        let mut metadata = Params::new();
        metadata.insert(FEED_ID_KEY.to_string(), json!(feed_id));
        DataType::new(
            CHAINLINK_DATA_TYPE,
            Some(metadata),
            Some(feed_id.to_string()),
        )
    }

    /// Returns metadata for serialization-oriented integrations.
    #[must_use]
    pub fn get_metadata(feed_id: &str) -> HashMap<String, String> {
        HashMap::from([(FEED_ID_KEY.to_string(), feed_id.to_string())])
    }

    /// Decodes a Chainlink WebSocket report into a `ChainlinkData`.
    ///
    /// # Errors
    ///
    /// Returns an error if the report payload cannot be decoded as a v3 report.
    pub fn from_ws_report(response: &WebSocketReport, ts_init: UnixNanos) -> anyhow::Result<Self> {
        let payload_hex = response
            .report
            .full_report
            .strip_prefix("0x")
            .unwrap_or(&response.report.full_report);
        let full_payload = hex::decode(payload_hex)
            .map_err(|e| anyhow::anyhow!("Failed to decode Chainlink full_report hex: {e}"))?;
        let (_context, payload_blob) = decode_full_report(&full_payload)
            .map_err(|e| anyhow::anyhow!("Failed to decode Chainlink full report payload: {e}"))?;
        let decoded = ReportDataV3::decode(&payload_blob)
            .map_err(|e| anyhow::anyhow!("Failed to decode Chainlink report payload as v3: {e}"))?;

        let feed_id = response.report.feed_id.to_hex_string();
        let observations_timestamp = response.report.observations_timestamp as u64;
        let ts_event = to_unix_nanos(observations_timestamp).as_u64();

        Ok(Self {
            feed_id,
            benchmark_price: decoded.benchmark_price.to_string(),
            bid: decoded.bid.to_string(),
            ask: decoded.ask.to_string(),
            ts_event,
            ts_init: ts_init.as_u64(),
        })
    }
}

fn to_unix_nanos(timestamp: u64) -> UnixNanos {
    let nanos = match timestamp {
        value if value >= 1_000_000_000_000_000_000 => value,
        value if value >= 1_000_000_000_000_000 => value * 1_000,
        value if value >= 1_000_000_000_000 => value * 1_000_000,
        value => value * 1_000_000_000,
    };
    UnixNanos::from(nanos)
}

impl HasTsInit for ChainlinkData {
    fn ts_init(&self) -> UnixNanos {
        UnixNanos::from(self.ts_init)
    }
}

impl CustomDataTrait for ChainlinkData {
    fn type_name(&self) -> &'static str {
        CHAINLINK_DATA_TYPE
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn ts_event(&self) -> UnixNanos {
        UnixNanos::from(self.ts_event)
    }

    fn to_json(&self) -> anyhow::Result<String> {
        Ok(serde_json::to_string(self)?)
    }

    fn clone_arc(&self) -> Arc<dyn CustomDataTrait> {
        Arc::new(self.clone())
    }

    fn eq_arc(&self, other: &dyn CustomDataTrait) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            self == other
        } else {
            false
        }
    }

    #[cfg(feature = "python")]
    fn to_pyobject(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<pyo3::Py<pyo3::PyAny>> {
        nautilus_model::data::custom::clone_pyclass_to_pyobject(self, py)
    }

    fn type_name_static() -> &'static str {
        CHAINLINK_DATA_TYPE
    }

    fn from_json(value: serde_json::Value) -> anyhow::Result<Arc<dyn CustomDataTrait>> {
        let parsed: Self = serde_json::from_value(value)?;
        Ok(Arc::new(parsed))
    }
}

#[cfg(test)]
mod tests {
    use chainlink_data_streams_report::{
        feed_id::ID,
        report::{Report, v3::ReportDataV3},
    };
    use chainlink_data_streams_sdk::stream::WebSocketReport;
    use num_bigint::BigInt;
    use rstest::rstest;

    use super::*;

    const FEED_ID: &str = "0x00036b4aa7e57ca7b68ae1bf45653f56b656fd3aa335ef7fae696b663f1b8472";
    const OBSERVATIONS_TS: u32 = 1_718_885_772;
    const EXPIRES_TS: u32 = 1_718_885_872;

    fn generate_mock_full_report(encoded_report_data: &[u8]) -> Vec<u8> {
        let mut payload = Vec::new();

        for _ in 0..3 {
            payload.extend_from_slice(&[0u8; 32]);
        }

        let mut offset = [0u8; 32];
        let offset_value: usize = 96 + 32;
        offset[24..32].copy_from_slice(&offset_value.to_be_bytes());
        payload.extend_from_slice(&offset);

        let mut length = [0u8; 32];
        let length_value: usize = encoded_report_data.len();
        length[24..32].copy_from_slice(&length_value.to_be_bytes());
        payload.extend_from_slice(&length);

        payload.extend_from_slice(encoded_report_data);
        payload
    }

    #[rstest]
    fn test_chainlink_data_type_uses_feed_metadata() {
        let data_type = ChainlinkData::data_type(FEED_ID);
        let metadata = data_type.metadata().expect("metadata must be present");
        assert_eq!(data_type.type_name(), CHAINLINK_DATA_TYPE);
        assert_eq!(metadata.get_str(FEED_ID_KEY), Some(FEED_ID));
        assert_eq!(data_type.identifier(), Some(FEED_ID));
    }

    #[rstest]
    fn test_chainlink_data_decodes_v3_report() {
        let feed_id = ID::from_hex_str(FEED_ID).unwrap();
        let report_data = ReportDataV3 {
            feed_id,
            valid_from_timestamp: OBSERVATIONS_TS,
            observations_timestamp: OBSERVATIONS_TS,
            native_fee: BigInt::from(10),
            link_fee: BigInt::from(20),
            expires_at: EXPIRES_TS,
            benchmark_price: BigInt::from(100),
            bid: BigInt::from(90),
            ask: BigInt::from(110),
        };
        let full_report = generate_mock_full_report(&report_data.abi_encode().unwrap());
        let ws_report = WebSocketReport {
            report: Report {
                feed_id,
                valid_from_timestamp: OBSERVATIONS_TS as usize,
                observations_timestamp: OBSERVATIONS_TS as usize,
                full_report: format!("0x{}", hex::encode(full_report)),
            },
        };

        let ts_init = UnixNanos::from(123);
        let data = ChainlinkData::from_ws_report(&ws_report, ts_init).unwrap();

        assert_eq!(data.feed_id, FEED_ID);
        assert_eq!(data.benchmark_price, "100");
        assert_eq!(data.bid, "90");
        assert_eq!(data.ask, "110");
        assert_eq!(data.ts_event, (OBSERVATIONS_TS as u64) * 1_000_000_000);
        assert_eq!(data.ts_init, ts_init.as_u64());
    }
}
