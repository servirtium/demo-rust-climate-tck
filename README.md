Note - The World Bank took down their climate WebAPI. Darn it. We now depend on a docker version of the same until we work out what to do long term. Docker build and deploy this locally - https://github.com/servirtium/worldbank-climate-recordings - see README

TL;DR:

```
docker build git@github.com:servirtium/worldbank-climate-recordings.git#main -t worldbank-api
docker run -d -p 4567:4567 worldbank-api
```

The build for this demo project needs that docker container running

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
