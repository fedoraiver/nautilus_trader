# -------------------------------------------------------------------------------------------------
#  Copyright (C) 2015-2026 Nautech Systems Pty Ltd. All rights reserved.
#  https://nautechsystems.io
#
#  Licensed under the GNU Lesser General Public License Version 3.0 (the "License");
#  You may not use this file except in compliance with the License.
#  You may obtain a copy of the License at https://www.gnu.org/licenses/lgpl-3.0.en.html
#
#  Unless required by applicable law or agreed to in writing, software
#  distributed under the License is distributed on an "AS IS" BASIS,
#  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
#  See the License for the specific language governing permissions and
#  limitations under the License.
# -------------------------------------------------------------------------------------------------

from nautilus_trader.adapters.chainlink import ChainlinkData
from nautilus_trader.adapters.chainlink import ChainlinkDataClientConfig
from nautilus_trader.adapters.chainlink import ChainlinkDataClientFactory
from nautilus_trader.common import Environment
from nautilus_trader.live import LiveNode
from nautilus_trader.model import TraderId
from nautilus_trader.model.data import DataType


FEED_ID = "0x00036b4aa7e57ca7b68ae1bf45653f56b656fd3aa335ef7fae696b663f1b8472"


def test_chainlink_adapter_imports_and_builder_path():
    config = ChainlinkDataClientConfig(feed_ids=[FEED_ID])
    factory = ChainlinkDataClientFactory()

    data_type = DataType(ChainlinkData, {"feed_id": FEED_ID}, FEED_ID)
    assert data_type.identifier == FEED_ID

    node = (
        LiveNode.builder("CHAINLINK-TEST", TraderId("TESTER-CHAINLINK"), Environment.SANDBOX)
        .add_data_client(None, factory, config)
        .build()
    )

    assert node.trader_id == TraderId("TESTER-CHAINLINK")
