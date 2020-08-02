use crate::error::Error;
use reqwest::{self};
type ReqwestClient = reqwest::blocking::Client;
use crate::data::annual_gcm_data::AnnualGcmData;

const DEFAULT_DOMAIN_NAME: &str = "http://climatedataapi.worldbank.org";

/// Builder used to build a ClimateApiClient instance
#[derive(Debug, Clone, Default)]
pub struct ClimateApiClientBuilder {
    domain_name: Option<String>,
    http_client: Option<ReqwestClient>,
}

impl ClimateApiClientBuilder {
    /// Create a new ClimateApiClientBuilder instance.
    pub fn new() -> Self {
        Self {
            domain_name: None,
            http_client: None,
        }
    }

    /// Use the given domain_name when building a ClimateApiClient instance.
    ///
    /// # Arguments
    /// `domain_name` - a domain name to use when calling the API.
    ///
    /// # Returns
    /// This builder.
    pub fn with_domain_name<T: Into<String>>(mut self, domain_name: T) -> Self {
        self.domain_name = Some(domain_name.into());
        self
    }

    /// Use the given blocking reqwest client when building a ClimateApiClient instance.
    ///
    /// # Arguments
    /// `client` - a pre-configured blocking reqwest client.
    ///
    /// # Returns
    /// This builder.
    pub fn with_http_client(mut self, client: ReqwestClient) -> Self {
        self.http_client = Some(client);
        self
    }

    /// Consume the builder and create a ClimateApiClient instance using all of the previously configured values or
    /// their defaults.
    ///
    /// # Returns
    /// A ClimateApiClient instance.
    pub fn build(mut self) -> ClimateApiClient {
        ClimateApiClient {
            http: self.http_client.take().unwrap_or_default(),
            domain_name: self
                .domain_name
                .take()
                .unwrap_or_else(|| String::from(DEFAULT_DOMAIN_NAME)),
        }
    }
}

/// Struct that represents a World Bank Climate Data API client.
#[derive(Default, Debug, Clone)]
pub struct ClimateApiClient {
    http: ReqwestClient,
    domain_name: String,
}

impl ClimateApiClient {
    /// Create a ClimateApiClient with the default reqwest client.
    ///
    /// # Returns
    /// A ClimateApiClient.
    pub fn new() -> Self {
        ClimateApiClient {
            http: ReqwestClient::new(),
            domain_name: String::from(DEFAULT_DOMAIN_NAME),
        }
    }

    /// Gets an average annual rainfall data from WorldBank Climate Data API.
    ///
    /// # Arguments
    /// `from_year` - start of the year interval. It should be a value between 1920 and 2080 inclusive and it should be
    ///     divisible by 20.
    /// `to_year` - end of the year interval. It should be a value equal to `from_year` + 19.
    /// `country_iso` - ISO3 country code
    ///
    /// # Returns
    /// Average of all of the average annual values from all Global Circulation Models (GCM).
    pub fn get_average_annual_rainfall<T: AsRef<str>>(
        &self,
        from_year: u16,
        to_year: u16,
        country_iso: T,
    ) -> Result<f64, Error> {
        let url = self.construct_get_average_annual_rainfall_url(from_year, to_year, country_iso);

        let response_text = self.http.get(&url).send()?.error_for_status()?.text()?;

        if response_text.starts_with("Invalid country code") {
            return Err(Error::NotRecognizedByClimateWeb);
        }

        let data: AnnualGcmData = quick_xml::de::from_str(&response_text)?;
        let data = match data.results {
            Some(data) => data,
            None => return Err(Error::DateRangeNotSupported(from_year, to_year)),
        };

        let (sum, count) = data.into_iter().fold((0.0, 0), |(sum, count), datum| {
            (sum + datum.annual_data.double, count + 1)
        });

        Ok(match count {
            0 => 0.0,
            _ => sum / count as f64,
        })
    }

    pub fn get_average_annual_rainfall_for_two<T1: AsRef<str>, T2: AsRef<str>>(
        &self,
        from_year: u16,
        to_year: u16,
        country_iso_first: T1,
        country_iso_second: T2,
    ) -> Result<(f64, f64), Error> {
        let first = self.get_average_annual_rainfall(from_year, to_year, country_iso_first)?;
        let second = self.get_average_annual_rainfall(from_year, to_year, country_iso_second)?;

        Ok((first, second))
    }

    fn construct_get_average_annual_rainfall_url<T: AsRef<str>>(
        &self,
        from_year: u16,
        to_year: u16,
        country_iso: T,
    ) -> String {
        format!(
            "{}/climateweb/rest/v1/country/annualavg/pr/{}/{}/{}.xml",
            self.domain_name,
            from_year,
            to_year,
            country_iso.as_ref()
        )
    }
    }

#[cfg(test)]
mod tests {
    use crate::{error::Error, ClimateApiClient, ClimateApiClientBuilder};
    use servirtium::{
        servirtium_playback_test, servirtium_record_test, Mutations, ServirtiumConfiguration,
    };

    fn servirtium_configure(config: &mut ServirtiumConfiguration) {
        config.set_domain_name("http://climatedataapi.worldbank.org");

        config.add_record_mutations(
            Mutations::new().remove_response_headers(vec!["set-cookie", "date"]),
        );
    }

    #[test]
    fn test_average_rainfall_for_great_britain_from_1980_to_1999_exists_direct() {
        test_average_rainfall_for_great_britain_from_1980_to_1999_exists(ClimateApiClient::new());
    }

    #[test]
    #[servirtium_playback_test(
        "playback_data/average_Rainfall_For_Great_Britain_From_1980_to_1999_Exists.md",
        servirtium_configure
    )]
    fn test_average_rainfall_for_great_britain_from_1980_to_1999_exists_playback() {
        test_average_rainfall_for_great_britain_from_1980_to_1999_exists(
            ClimateApiClientBuilder::new()
                .with_domain_name("http://localhost:61417")
                .build(),
        );
    }

    #[test]
    #[servirtium_record_test(
        "playback_data/average_Rainfall_For_Great_Britain_From_1980_to_1999_Exists.md",
        servirtium_configure
    )]
    fn test_average_rainfall_for_great_britain_from_1980_to_1999_exists_record() {
        test_average_rainfall_for_great_britain_from_1980_to_1999_exists(
            ClimateApiClientBuilder::new()
                .with_domain_name("http://localhost:61417")
                .build(),
        );
    }

    fn test_average_rainfall_for_great_britain_from_1980_to_1999_exists(
        climate_api: ClimateApiClient,
    ) {
        assert!(
            climate_api
                .get_average_annual_rainfall(1980, 1999, "gbr")
                .unwrap()
                - 988.8454972331015
                < f64::EPSILON
        );
    }

    #[test]
    fn test_average_rainfall_for_france_from_1980_to_1999_exists_direct() {
        test_average_rainfall_for_france_from_1980_to_1999_exists(ClimateApiClient::new());
    }

    #[test]
    #[servirtium_playback_test(
        "playback_data/average_Rainfall_For_France_From_1980_to_1999_Exists.md",
        servirtium_configure
    )]
    fn test_average_rainfall_for_france_from_1980_to_1999_exists_playback() {
        test_average_rainfall_for_france_from_1980_to_1999_exists(
            ClimateApiClientBuilder::new()
                .with_domain_name("http://localhost:61417")
                .build(),
        );
    }

    #[test]
    #[servirtium_record_test(
        "playback_data/average_Rainfall_For_France_From_1980_to_1999_Exists.md",
        servirtium_configure
    )]
    fn test_average_rainfall_for_france_from_1980_to_1999_exists_record() {
        test_average_rainfall_for_france_from_1980_to_1999_exists(
            ClimateApiClientBuilder::new()
                .with_domain_name("http://localhost:61417")
                .build(),
        );
    }

    fn test_average_rainfall_for_france_from_1980_to_1999_exists(climate_api: ClimateApiClient) {
        assert!(
            climate_api
                .get_average_annual_rainfall(1980, 1999, "fra")
                .unwrap()
                - 913.7986955122727
                < f64::EPSILON
        );
    }

    #[test]
    fn test_average_rainfall_for_egypt_from_1980_to_1999_exists_direct() {
        test_average_rainfall_for_egypt_from_1980_to_1999_exists(ClimateApiClient::new());
    }

    #[test]
    #[servirtium_playback_test(
        "playback_data/average_Rainfall_For_Egypt_From_1980_to_1999_Exists.md",
        servirtium_configure
    )]
    fn test_average_rainfall_for_egypt_from_1980_to_1999_exists_playback() {
        test_average_rainfall_for_egypt_from_1980_to_1999_exists(
            ClimateApiClientBuilder::new()
                .with_domain_name("http://localhost:61417")
                .build(),
        );
    }

    #[test]
    #[servirtium_record_test(
        "playback_data/average_Rainfall_For_Egypt_From_1980_to_1999_Exists.md",
        servirtium_configure
    )]
    fn test_average_rainfall_for_egypt_from_1980_to_1999_exists_record() {
        test_average_rainfall_for_egypt_from_1980_to_1999_exists(
            ClimateApiClientBuilder::new()
                .with_domain_name("http://localhost:61417")
                .build(),
        );
    }

    fn test_average_rainfall_for_egypt_from_1980_to_1999_exists(climate_api: ClimateApiClient) {
        assert!(
            climate_api
                .get_average_annual_rainfall(1980, 1999, "egy")
                .unwrap()
                - 54.58587712129825
                < f64::EPSILON
        );
    }

    #[test]
    fn test_average_rainfall_for_great_britain_from_1985_to_1995_does_not_exist_direct() {
        test_average_rainfall_for_great_britain_from_1985_to_1995_does_not_exist(
            ClimateApiClient::new(),
        );
    }

    #[test]
    #[servirtium_playback_test(
        "playback_data/average_Rainfall_For_Great_Britain_From_1985_to_1995_Does_Not_Exist.md",
        servirtium_configure
    )]
    fn test_average_rainfall_for_great_britain_from_1985_to_1995_does_not_exist_playback() {
        test_average_rainfall_for_great_britain_from_1985_to_1995_does_not_exist(
            ClimateApiClientBuilder::new()
                .with_domain_name("http://localhost:61417")
                .build(),
        );
    }

    #[test]
    #[servirtium_record_test(
        "playback_data/average_Rainfall_For_Great_Britain_From_1985_to_1995_Does_Not_Exist.md",
        servirtium_configure
    )]
    fn test_average_rainfall_for_great_britain_from_1985_to_1995_does_not_exist_record() {
        test_average_rainfall_for_great_britain_from_1985_to_1995_does_not_exist(
            ClimateApiClientBuilder::new()
                .with_domain_name("http://localhost:61417")
                .build(),
        );
    }

    fn test_average_rainfall_for_great_britain_from_1985_to_1995_does_not_exist(
        climate_api: ClimateApiClient,
    ) {
        let result = climate_api.get_average_annual_rainfall(1985, 1995, "gbr");

        match result {
            Err(err) => match err {
                Error::DateRangeNotSupported(1985, 1995) => (),
                _ => panic!("The function returned a wrong error: {}", err.to_string()),
            },
            _ => panic!("The function call should return an error"),
        }
    }

    #[test]
    fn test_average_rainfall_for_middle_earth_from_1980_to_1999_does_not_exist_direct() {
        test_average_rainfall_for_middle_earth_from_1980_to_1999_does_not_exist(
            ClimateApiClient::new(),
        );
    }

    #[test]
    #[servirtium_playback_test(
        "playback_data/average_Rainfall_For_Middle_Earth_From_1980_to_1999_Does_Not_Exist.md",
        servirtium_configure
    )]
    fn test_average_rainfall_for_middle_earth_from_1980_to_1999_does_not_exist_playback() {
        test_average_rainfall_for_middle_earth_from_1980_to_1999_does_not_exist(
            ClimateApiClientBuilder::new()
                .with_domain_name("http://localhost:61417")
                .build(),
        );
    }

    #[test]
    #[servirtium_record_test(
        "playback_data/average_Rainfall_For_Middle_Earth_From_1980_to_1999_Does_Not_Exist.md",
        servirtium_configure
    )]
    fn test_average_rainfall_for_middle_earth_from_1980_to_1999_does_not_exist_record() {
        test_average_rainfall_for_middle_earth_from_1980_to_1999_does_not_exist(
            ClimateApiClientBuilder::new()
                .with_domain_name("http://localhost:61417")
                .build(),
        );
    }

    fn test_average_rainfall_for_middle_earth_from_1980_to_1999_does_not_exist(
        climate_api: ClimateApiClient,
    ) {
        let result = climate_api.get_average_annual_rainfall(1980, 1999, "mde");

        match result {
            Err(err) => match err {
                Error::NotRecognizedByClimateWeb => (),
                _ => panic!("The function returned a wrong error: {}", err.to_string()),
            },
            _ => panic!("The function call should return an error"),
        }
    }

    #[test]
    fn test_average_rainfall_for_great_britain_and_france_from_1980_to_1999_exist_direct() {
        test_average_rainfall_for_great_britain_and_france_from_1980_to_1999_exist(
            ClimateApiClient::new(),
        );
    }

    #[test]
    #[servirtium_playback_test(
        "playback_data/average_Rainfall_For_Great_Britain_And_France_From_1980_to_1999_Exists.md",
        servirtium_configure
    )]
    fn test_average_rainfall_for_great_britain_and_france_from_1980_to_1999_exist_playback() {
        test_average_rainfall_for_great_britain_and_france_from_1980_to_1999_exist(
            ClimateApiClientBuilder::new()
                .with_domain_name("http://localhost:61417")
                .build(),
        );
    }

    #[test]
    #[servirtium_record_test(
        "playback_data/average_Rainfall_For_Great_Britain_And_France_From_1980_to_1999_Exists.md",
        servirtium_configure
    )]
    fn test_average_rainfall_for_great_britain_and_france_from_1980_to_1999_exist_record() {
        test_average_rainfall_for_great_britain_and_france_from_1980_to_1999_exist(
            ClimateApiClientBuilder::new()
                .with_domain_name("http://localhost:61417")
                .build(),
        );
    }

    fn test_average_rainfall_for_great_britain_and_france_from_1980_to_1999_exist(
        climate_api: ClimateApiClient,
    ) {
        let (gbr, fra) = climate_api
            .get_average_annual_rainfall_for_two(1980, 1999, "gbr", "fra")
            .unwrap();

        assert!(gbr - 988.8454972331015 < f64::EPSILON);
        assert!(fra - 913.7986955122727 < f64::EPSILON);
    }
}
