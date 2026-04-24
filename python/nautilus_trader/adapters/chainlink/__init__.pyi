# This file is manually maintained until pyo3_stub_gen includes the Chainlink adapter.

import typing

__all__ = [
    "ChainlinkData",
    "ChainlinkDataClientConfig",
    "ChainlinkDataClientFactory",
]

@typing.final
class ChainlinkData:
    @property
    def feed_id(self) -> str: ...
    @property
    def benchmark_price(self) -> str: ...
    @property
    def bid(self) -> str: ...
    @property
    def ask(self) -> str: ...
    @property
    def ts_event(self) -> int: ...
    @property
    def ts_init(self) -> int: ...

@typing.final
class ChainlinkDataClientConfig:
    def __init__(
        self,
        api_key: str | None = None,
        api_secret: str | None = None,
        api_base_url: str | None = None,
        ws_base_url: str | None = None,
        feed_ids: list[str] | None = None,
        reconnect_delay_secs: float = 5.0,
        ws_high_availability: bool = False,
        ws_max_reconnect: int = 5,
        insecure_skip_verify: bool = False,
    ) -> None: ...

@typing.final
class ChainlinkDataClientFactory:
    def __init__(self) -> None: ...
    def name(self) -> str: ...
