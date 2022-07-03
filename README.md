# selecting

[![Rust](https://github.com/DoumanAsh/selecting/actions/workflows/rust.yml/badge.svg)](https://github.com/DoumanAsh/selecting/actions/workflows/rust.yml)
[![Crates.io](https://img.shields.io/crates/v/selecting.svg)](https://crates.io/crates/selecting)
[![Documentation](https://docs.rs/selecting/badge.svg)](https://docs.rs/crate/selecting/)

Cross-platform wrapper over select.

This library provides simple interface over POSIX's `select` enabling you to write
very simple async programs using `std` networking primitives.

But if you want performance you should look for `tokio` or `mio`
