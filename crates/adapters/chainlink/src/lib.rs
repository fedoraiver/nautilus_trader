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

//! [NautilusTrader](https://nautilustrader.io) adapter for
//! [Chainlink Data Streams](https://docs.chain.link/data-streams).
//!
//! The `nautilus-chainlink` crate provides a live data-only integration that
//! converts Chainlink Data Streams reports into Nautilus custom data events.
//! Reports are subscribed by `feed_id` and emitted as [`crate::types::ChainlinkData`].
//!
//! # Feature Flags
//!
//! - `live` (default): Enables the live data client and factory.
//! - `python`: Enables Python bindings from [PyO3](https://pyo3.rs).
//! - `extension-module`: Builds as a Python extension module.
//! - `high-precision`: Enables high-precision mode in dependent Nautilus types.

#![warn(rustc::all)]
#![deny(unsafe_code)]
#![deny(nonstandard_style)]
#![deny(missing_debug_implementations)]
#![deny(clippy::missing_errors_doc)]
#![deny(clippy::missing_panics_doc)]
#![deny(rustdoc::broken_intra_doc_links)]

pub mod common;
pub mod config;
pub mod types;

#[cfg(feature = "live")]
pub mod data;

#[cfg(feature = "live")]
pub mod factories;

#[cfg(feature = "python")]
pub mod python;
