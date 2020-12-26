# serbuffer

[![Crates.io](https://img.shields.io/crates/v/serbuffer?color=blue)](https://crates.io/crates/serbuffer)
[![Released API docs](https://docs.rs/serbuffer/badge.svg)](https://docs.rs/serbuffer)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE-MIT)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](./LICENSE-APACHE)

Base on `bytes`, can work well with `tokio`.
It allows you to directly access serialized data without parsing/unpacking it first.
Especially in streaming computing scenarios, networks and operators interact with zero copies of direct data.

Framework tested on Linux/MacOS/Windows, requires stable Rust.
