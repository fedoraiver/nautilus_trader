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
"""
Configuration wrappers for the PyO3-backed Chainlink adapter.
"""

from __future__ import annotations

from typing import TYPE_CHECKING

from nautilus_trader.config import RoutingConfig
from nautilus_trader.core import nautilus_pyo3


if not hasattr(nautilus_pyo3, "chainlink"):
    raise ImportError(
        "Chainlink PyO3 bindings are not available. Rebuild the nautilus_pyo3 extension "
        "with the Chainlink adapter enabled.",
    )

if TYPE_CHECKING:

    class RustChainlinkDataClientConfig:
        def __new__(
            cls,
            api_key: str | None = None,
            api_secret: str | None = None,
            api_base_url: str | None = None,
            ws_base_url: str | None = None,
            feed_ids: list[str] | None = None,
            reconnect_delay_secs: float = 5.0,
            ws_high_availability: bool = False,
            ws_max_reconnect: int = 5,
            insecure_skip_verify: bool = False,
        ): ...
else:
    RustChainlinkDataClientConfig = nautilus_pyo3.chainlink.ChainlinkDataClientConfig


class ChainlinkDataClientConfig(RustChainlinkDataClientConfig):
    """
    Configuration for the Rust Chainlink data client.

    This wrapper exists so downstream Python code can import a stable adapter
    path from ``nautilus_trader.adapters.chainlink`` while still using the Rust
    ``LiveNode.builder(...).add_data_client(...)`` integration.

    Notes
    -----
    This is not a ``LiveDataClientConfig`` for the legacy Python
    ``TradingNode(data_clients=...)`` path.

    """

    def __new__(
        cls,
        api_key: str | None = None,
        api_secret: str | None = None,
        api_base_url: str | None = None,
        ws_base_url: str | None = None,
        feed_ids: list[str] | None = None,
        reconnect_delay_secs: float = 5.0,
        ws_high_availability: bool = False,
        ws_max_reconnect: int = 5,
        insecure_skip_verify: bool = False,
        routing: RoutingConfig | None = None,
        **kwargs,
    ):
        obj = super().__new__(
            cls,
            api_key=api_key,
            api_secret=api_secret,
            api_base_url=api_base_url,
            ws_base_url=ws_base_url,
            feed_ids=feed_ids,
            reconnect_delay_secs=reconnect_delay_secs,
            ws_high_availability=ws_high_availability,
            ws_max_reconnect=ws_max_reconnect,
            insecure_skip_verify=insecure_skip_verify,
        )
        # Retained as a plain Python attribute so callers can keep a consistent
        # config shape across adapters when they inspect routing metadata.
        obj.routing = routing if routing is not None else RoutingConfig()

        if kwargs:
            unknown = ", ".join(sorted(kwargs))
            raise TypeError(f"Unexpected ChainlinkDataClientConfig kwargs: {unknown}")

        return obj
