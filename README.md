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

## Usage

Use `classicl_server --help` for usage information, running without providing
any flags will use default values.

## Building

With cargo being installed run

```console
cargo build
```

## License

This project is licensed under the AGPL-3.0-or-later license.

classicl-server Copyright (C) 2022 nokoe

This program is free software: you can redistribute it and/or modify it under
the terms of the GNU Affero General Public License as published by the Free
Software Foundation, either version 3 of the License, or (at your option) any
later version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY
WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A
PARTICULAR PURPOSE. See the GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License along
with this program. If not, see <https://www.gnu.org/licenses/>.
