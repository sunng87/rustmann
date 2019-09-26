# rustmann

[![Crates.io](https://img.shields.io/crates/v/rustmann.svg)](https://crates.io/crates/rustmann)
[![Docs](https://docs.rs/rustmann/badge.svg)](https://docs.rs/crate/rustmann/)
[![Build Status](https://travis-ci.org/sunng87/rustmann.svg?branch=master)](https://travis-ci.org/sunng87/rustmann)
![GitHub](https://img.shields.io/github/license/sunng87/rustmann.svg)
[![Donate](https://img.shields.io/badge/donate-liberapay-yellow.svg)](https://liberapay.com/Sunng/donate)

A [riemann](https://riemann.io/) client using
[tokio](https://tokio.rs). This project is still in its early
stage and API changes aggressively.

## Usage

See
[examples](https://github.com/sunng87/rustmann/tree/master/examples)
and [API docs](https://docs.rs/crate/rustmann/).

## Features & TODOs

- [x] TCP Client
- [x] TLS TCP Client (by enabling `tls` feature)
- [x] UDP Client
- [x] Report API (`send_events`)
- [x] Query API (`send_query`)
- [x] Event Builder API

## Minimal version policy

This library relies heavily on async-await feature so it requires Rust
1.39 and above to compile.

## License

MIT/Apache-2.0
