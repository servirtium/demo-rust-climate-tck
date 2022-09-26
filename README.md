Note - The World Bank took down their climate WebAPI on http://climatedataapi.worldbank.org

Darn it. We now depend on https://servirtium.github.io/worldbank-climate-recordings for the same endpoints.

# WorldBank Climate Data Api Client

A simple API client for retrieving data from the World Bank Climate Data Api.
Currently, it supports only getting average annual rainfall values.

## Prerequisites

To build the project, you have to install **Rust** first.  The Rust
installation should include _cargo_ tool. The only tested version is 1.44.1.

## How to build

`$ cargo build`

## How to run tests

`$ cargo test` - 18 tests

### Running a subset of the tests

To run only direct tests, do `cargo test direct` - 6 tests

Only playback tests, do `cargo test playback` - 6 tests

And only record tests, do `cargo test record` - 6 tests

## License

Licensed under MIT License ([LICENSE](LICENSE) or
http://opensource.org/licenses/MIT)
