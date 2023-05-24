# classicl_server

## Introduction

This is a Minecraft Classic server written in Rust powered by
[tokio](https://tokio.rs/) and [classicl](https://gitlab.com/nokoe/classicl).

## Installation

### Cargo

The project can be installed using Cargo through the following steps:

- Install the Rust toolchain using the
  [official guide](https://www.rust-lang.org/tools/install).
- Run `cargo install --git https://gitlab.com/nokoe/classicl-server`

You can also download pre-built binaries from [Github Releases](https://github.com/6e6f6b6f65/classicl-server/releases).

## Usage

Use `classicl_server --help` for usage information, running without providing
any flags will use default values.

## Building

With cargo being installed run

```console
cargo build
```

## License

This project is licensed under the AGPL-3.0-or-later. See [COPYING](COPYING) or
the head of the source files for more information.