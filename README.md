# WorldBank Climate Data Api Client

A simple API client for retreiving data from the World Bacnk Climate Data Api.
Currently it supports only getting average annual rainfall values.

## Prerequisites

To build the project, you have to install **Rust** first.  The Rust
installation should include _cargo_ tool. The only tested version is 1.44.1.

## How to build

`$ cargo build`

## How to run tests

`$ cargo test`

### Running a subset of the tests

To run only direct tests, do `cargo test direct`

Only playback tests, do `cargo test playback` 

And only record tests, do `cargo test record`

## License

Licensed under MIT License ([LICENSE](LICENSE) or
http://opensource.org/licenses/MIT)