# rustmann

[![Crates.io](https://img.shields.io/crates/v/rustmann.svg)](https://crates.io/crates/rustmann)
[![Docs](https://docs.rs/handlebars/badge.svg)](https://docs.rs/crate/handlebars/)
[![Build Status](https://travis-ci.org/sunng87/rustmann.svg?branch=master)](https://travis-ci.org/sunng87/rustmann)
![GitHub](https://img.shields.io/github/license/sunng87/rustmann.svg)
[![Donate](https://img.shields.io/badge/donate-liberapay-yellow.svg)](https://liberapay.com/Sunng/donate)

A [riemann](https://riemann.io/) client with
[tokio](https://tokio.rs). This project is still in its early
stage and API changes aggressively. Note that this library is written
in async/await style (due to [async borrow
issue](http://aturon.github.io/2018/04/24/async-borrowing/)) so for
now it compiles on nightly rust only.

## Usage

See
[examples](https://github.com/sunng87/rustmann/tree/master/examples)
and [API docs](https://docs.rs/crate/rustmann/).

## License

MIT/Apache-2.0
