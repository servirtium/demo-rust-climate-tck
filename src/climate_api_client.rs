use crate::error::Error;
use reqwest::{self};
type ReqwestClient = reqwest::blocking::Client;
use crate::data::annual_gcm_data::AnnualGcmData;
use quick_xml;

const DEFAULT_DOMAIN_NAME: &'static str = "http://climatedataapi.worldbank.org/";

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
            http: self.http_client.take().unwrap_or(ReqwestClient::new()),
            domain_name: self
                .domain_name
                .take()
                .unwrap_or(String::from(DEFAULT_DOMAIN_NAME)),
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
        Self::check_years(from_year, to_year)?;

        let url = self.construct_get_average_annual_rainfall_url(from_year, to_year, country_iso);

        let response_text = self.http.get(&url).send()?.error_for_status()?.text()?;

        if response_text.starts_with("Invalid country code") {
            return Err(Error::NotRecognizedByClimateWeb);
        }

        let data: AnnualGcmData = quick_xml::de::from_str(&response_text)?;
        let data = data.results.unwrap_or(Vec::new());

        let (sum, count) = data.into_iter().fold((0.0, 0), |(sum, count), datum| {
            (sum + datum.annual_data.double, count + 1)
        });

        Ok(match count {
            0 => 0.0,
            _ => sum / count as f64,
        })
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

    fn check_years(from_year: u16, to_year: u16) -> Result<(), Error> {
        if from_year < 1920 || from_year > 2080 || from_year % 20 != 0 || to_year != from_year + 19
        {
            Err(Error::DateRangeNotSupported(from_year, to_year))
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use crate::{error::Error, ClimateApiClient, ClimateApiClientBuilder};
    use lazy_static::lazy_static;
    use regex::Regex;
    use std::{fs, thread};
    use std::{
        io::{BufRead, BufReader, Write},
        net::{TcpListener, TcpStream},
        path::Path,
        sync::Once,
    };

    #[test]
    fn test_averageRainfallForGreatBritainFrom1980to1999Exists() {
        let climateApi = ClimateApiClient::new();
        assert_eq!(
            climateApi
                .get_average_annual_rainfall(1980, 1999, "gbr")
                .unwrap(),
            988.8454972331015
        );
    }

    #[test]
    fn test_averageRainfallForFranceFrom1980to1999Exists() {
        let climateApi = ClimateApiClient::new();
        assert_eq!(
            climateApi
                .get_average_annual_rainfall(1980, 1999, "fra")
                .unwrap(),
            913.7986955122727
        );
    }

    #[test]
    fn test_averageRainfallForEgyptFrom1980to1999Exists() {
        let climateApi = ClimateApiClient::new();
        assert_eq!(
            climateApi
                .get_average_annual_rainfall(1980, 1999, "egy")
                .unwrap(),
            54.58587712129825
        );
    }

    #[test]
    fn test_averageRainfallForGreatBritainFrom1985to1995DoesNotExist() {
        let climateApi = ClimateApiClient::new();
        let result = climateApi.get_average_annual_rainfall(1985, 1995, "gbr");

        match result {
            Err(err) => match err {
                Error::DateRangeNotSupported(1985, 1995) => (),
                _ => panic!("The function returned a wrong error: {}", err.to_string()),
            },
            _ => panic!("The function call should return an error"),
        }
    }

    #[test]
    fn test_averageRainfallForMiddleEarthFrom1980to1999DoesNotExist() {
        let climateApi = ClimateApiClient::new();
        let result = climateApi.get_average_annual_rainfall(1980, 1999, "mde");

        match result {
            Err(err) => match err {
                Error::NotRecognizedByClimateWeb => (),
                _ => panic!("The function returned a wrong error: {}", err.to_string()),
            },
            _ => panic!("The function call should return an error"),
        }
    }

    #[test]
    fn test_averageRainfallForGreatBritainFrom1999to1980DoesNotExist() {
        let climateApi = ClimateApiClient::new();
        let result = climateApi.get_average_annual_rainfall(1999, 1980, "gbr");

        match result {
            Err(err) => match err {
                Error::DateRangeNotSupported(1999, 1980) => (),
                _ => panic!("The function returned a wrong error: {}", err.to_string()),
            },
            _ => panic!("The function call should return an error"),
        }
    }

    #[test]
    fn test_averageRainfallForGreatBritainFrom1980to1999Exists_playback() {
        setup_playback_server();

        let climateApi = ClimateApiClientBuilder::new()
            .with_domain_name("http://localhost:61417")
            .build();

        assert_eq!(
            climateApi
                .get_average_annual_rainfall(1980, 1999, "gbr")
                .unwrap(),
            988.8454972331015
        );
    }

    #[test]
    fn test_averageRainfallForFranceFrom1980to1999Exists_playback() {
        setup_playback_server();

        let climateApi = ClimateApiClientBuilder::new()
            .with_domain_name("http://localhost:61417")
            .build();

        assert_eq!(
            climateApi
                .get_average_annual_rainfall(1980, 1999, "fra")
                .unwrap(),
            913.7986955122727
        );
    }

    #[test]
    fn test_averageRainfallForEgyptFrom1980to1999Exists_playback() {
        setup_playback_server();

        let climateApi = ClimateApiClientBuilder::new()
            .with_domain_name("http://localhost:61417")
            .build();

        assert_eq!(
            climateApi
                .get_average_annual_rainfall(1980, 1999, "egy")
                .unwrap(),
            54.58587712129825
        );
    }

    #[test]
    fn test_averageRainfallForGreatBritainFrom1985to1995DoesNotExist_playback() {
        setup_playback_server();

        let climateApi = ClimateApiClientBuilder::new()
            .with_domain_name("http://localhost:61417")
            .build();

        let result = climateApi.get_average_annual_rainfall(1985, 1995, "gbr");

        match result {
            Err(err) => match err {
                Error::DateRangeNotSupported(1985, 1995) => (),
                _ => panic!("The function returned a wrong error: {}", err.to_string()),
            },
            _ => panic!("The function call should return an error"),
        }
    }

    #[test]
    fn test_averageRainfallForMiddleEarthFrom1980to1999DoesNotExist_playback() {
        setup_playback_server();

        let climateApi = ClimateApiClientBuilder::new()
            .with_domain_name("http://localhost:61417")
            .build();

        let result = climateApi.get_average_annual_rainfall(1980, 1999, "mde");

        match result {
            Err(err) => match err {
                Error::NotRecognizedByClimateWeb => (),
                _ => panic!("The function returned a wrong error: {}", err.to_string()),
            },
            _ => panic!("The function call should return an error"),
        }
    }

    #[test]
    fn test_averageRainfallForGreatBritainFrom1999to1980DoesNotExist_playback() {
        setup_playback_server();

        let climateApi = ClimateApiClientBuilder::new()
            .with_domain_name("http://localhost:61417")
            .build();

        let result = climateApi.get_average_annual_rainfall(1999, 1980, "gbr");

        match result {
            Err(err) => match err {
                Error::DateRangeNotSupported(1999, 1980) => (),
                _ => panic!("The function returned a wrong error: {}", err.to_string()),
            },
            _ => panic!("The function call should return an error"),
        }
    }

    // reading all of the .md files is an expensive operation. That's why here is used the lazy_static crate.
    // It initializes a static variable on access and only once even if it's accessed concurrently.
    // It is important to do this using lazy_static and outside of call_once because call_once will not wait for the
    // callback to be done. If the callback has been launched by another thread but not yet completed, the code will
    // not wait and will send requests to nothing.
    lazy_static! {
        static ref PLAYBACK_LOADER: PlaybackLoader = {
            let mut playback_loader = PlaybackLoader::new();
            let markdown_files_iter = fs::read_dir("./playback_data")
                .expect("Couldn't read the playback_data directory")
                .into_iter()
                .map(|e| {
                    String::from(
                        e.expect("Couldn't read the playback_data directory entry")
                            .path()
                            .to_string_lossy(),
                    )
                })
                .filter(|e| e.ends_with(".md"));

            for dir in markdown_files_iter {
                playback_loader.add_from_markdown_file(dir);
            }

            playback_loader
        };
    }

    lazy_static! {
        static ref HEADER_REGEX: Regex =
            Regex::new(r"(?m)(?P<header_key>[a-zA-Z\-]+): (?P<header_value>.*?)$").unwrap();
    }

    lazy_static! {
        static ref MARKDOWN_REGEX: Regex = Regex::new(
                "(?ms)\\#\\# [^/]*(?P<uri>.*\\.xml).*?\\#\\#\\# Response headers recorded for playback.*?```\
                \\s*(?P<headers_part>.*?)\\s*```.*?\\#\\#\\# Response body recorded for playback.*?```\\s*\
                (?P<body_part>.*?)\\s*```.*?")
            .unwrap();
    }

    static TEST_INIT: Once = Once::new();

    fn setup_playback_server() {
        TEST_INIT.call_once(move || {
            thread::spawn(move || {
                let listener = TcpListener::bind("localhost:61417").unwrap();

                for stream in listener.incoming() {
                    match stream {
                        Ok(s) => {
                            handle_connection(s, &PLAYBACK_LOADER);
                        }
                        // in case of an error - shutdown the test server
                        Err(_) => break,
                    }
                }
            });
        });
    }

    fn handle_connection(mut stream: TcpStream, playback_loader: &PlaybackLoader) {
        let uri = read_uri_from_request(&stream);

        let playback_data = playback_loader
            .get_playback_data(uri)
            .expect("A markdown file for the given uri hasn't been found");

        let response = if playback_data.headers.is_empty() {
            format!("HTTP/1.1 200 OK\r\n\r\n{}", playback_data.response_body)
        } else {
            let headers = playback_data
                .headers
                .iter()
                // Transfer-Encoding: chunked shouldn't be included in local tests because all the data is
                // written immediately and reqwest panics because of that
                .filter(|(key, value)| key != "Transfer-Encoding" || value != "chunked")
                .map(|(key, value)| format!("{}: {}\r\n", key, value))
                .collect::<Vec<_>>()
                .join("");

            format!(
                "HTTP/1.1 200 OK\r\n{}\r\n{}",
                headers, playback_data.response_body
            )
        };

        stream
            .write(response.as_bytes())
            .expect("Couldn't write the response");
        stream.flush().expect("Couldn't flush the stream buffer");
    }

    fn read_uri_from_request(stream: &TcpStream) -> String {
        let mut request_line = String::new();
        let mut reader = BufReader::new(stream);
        reader
            .read_line(&mut request_line)
            .expect("The request should contain atleast the first line");

        request_line
            .split(' ')
            .skip(1)
            .next()
            .expect("The first line of the request should contain an URI")
            .to_string()
    }

    #[derive(Debug, Clone)]
    struct PlaybackLoader {
        playback_data: Vec<PlaybackData>,
    }

    impl PlaybackLoader {
        fn new() -> Self {
            Self {
                playback_data: Vec::new(),
            }
        }

        fn get_playback_data<T: AsRef<str>>(&self, uri: T) -> Option<&PlaybackData> {
            self.playback_data.iter().find(|d| d.uri == uri.as_ref())
        }

        fn add_from_markdown_file<T: AsRef<Path>>(&mut self, filename: T) {
            let file_contents =
                fs::read_to_string(filename).expect("Couldn't read the markdown file");

            let markdown_captures = MARKDOWN_REGEX
                .captures(&file_contents)
                .expect("The markdown file has wrong format");

            let uri = &markdown_captures["uri"];
            let headers_part = &markdown_captures["headers_part"];
            let body_part = &markdown_captures["body_part"];

            let headers = Self::parse_headers(headers_part);

            self.playback_data.push(PlaybackData {
                headers,
                response_body: String::from(body_part),
                uri: String::from(uri),
            });
        }

        fn parse_headers<T: AsRef<str>>(headers_part: T) -> Vec<(String, String)> {
            let mut headers = Vec::new();

            for capture in HEADER_REGEX.captures_iter(headers_part.as_ref()) {
                headers.push((
                    String::from(&capture["header_key"]),
                    String::from(&capture["header_value"]),
                ));
            }

            headers
        }
    }

    #[derive(Debug, Clone)]
    struct PlaybackData {
        pub uri: String,
        pub headers: Vec<(String, String)>,
        pub response_body: String,
    }
}
