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
Factory bindings for the PyO3-backed Chainlink adapter.

The exported factory is the Rust PyO3 factory object consumed by
``LiveNode.builder(...).add_data_client(...)``.

"""

from nautilus_trader.core import nautilus_pyo3


if not hasattr(nautilus_pyo3, "chainlink"):
    raise ImportError(
        "Chainlink PyO3 bindings are not available. Rebuild the nautilus_pyo3 extension "
        "with the Chainlink adapter enabled.",
    )

ChainlinkDataClientFactory = nautilus_pyo3.chainlink.ChainlinkDataClientFactory


__all__ = ["ChainlinkDataClientFactory"]
