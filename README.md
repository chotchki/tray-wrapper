# tray-wrapper
A simple wrapper library to make it easy to run servers with a GUI tray icon

[![Build and Test](https://github.com/chotchki/tray-wrapper/actions/workflows/build_test.yml/badge.svg)](https://github.com/chotchki/tray-wrapper/actions/workflows/build_test.yml) [![codecov](https://codecov.io/gh/chotchki/tray-wrapper/graph/badge.svg?token=JS8FF39SX5)](https://codecov.io/gh/chotchki/tray-wrapper)

## Status

The core of the library is functioning, however the main trait your code needs to implement is somewhat compromised. This library really needs generators and/or coroutines in Rust to be stabilized and as a result this library will not reach 1.0.0 until that occurs (see tracking issue here: https://github.com/rust-lang/rust/issues/43122).

All that being said, the main test / example in (ui_tests/main.rs) is still too leaky an abstraction and will go through major changes again.

## License

This work is dual-licensed under Apache 2.0 and MIT license.
You can choose between one of them if you use this work.

`SPDX-License-Identifier: Apache-2.0 OR MIT`
