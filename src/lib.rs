mod climate_api_client;
mod data;
mod error;
#[allow(dead_code)]
mod markdown_manager;
#[allow(dead_code)]
mod servirtium_error;
#[allow(dead_code)]
mod servirtium_server;

pub use climate_api_client::ClimateApiClient;
pub use climate_api_client::ClimateApiClientBuilder;
