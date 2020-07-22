use crate::error::Error;
use reqwest::{self};
type ReqwestClient = reqwest::blocking::Client;
use crate::data::annual_gcm_data::AnnualGcmData;
use quick_xml;

/// Struct that represents a World Bank Climate Data API client.
#[derive(Default, Debug, Clone)]
pub struct ClimateApiClient {
    http: ReqwestClient,
}

impl ClimateApiClient {
    /// Create a ClimateApiClient with the default reqwest client.
    ///
    /// # Returns
    /// A ClimateApiClient.
    pub fn new() -> Self {
        ClimateApiClient {
            http: ReqwestClient::new(),
        }
    }

    /// Create a `ClimateApiClient` and use the passed Reqwest client as a http client.
    ///
    /// # Arguments
    /// `client` - the reqwest client to use.
    ///
    /// # Returns
    ///
    /// A ClimateApiClient.
    pub fn with_client(client: ReqwestClient) -> Self {
        ClimateApiClient { http: client }
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

        let url = Self::construct_get_average_annual_rainfall_url(from_year, to_year, country_iso);

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
        from_year: u16,
        to_year: u16,
        country_iso: T,
    ) -> String {
        format!(
            "http://climatedataapi.worldbank.org/climateweb/rest/v1/country/annualavg/pr/{}/{}/{}.xml",
            from_year, to_year, country_iso.as_ref())
    }

    fn check_years(from_year: u16, to_year: u16) -> Result<(), Error> {
        if from_year < 1920
            || from_year > 2080
            || from_year % 20 != 0
            || to_year < 1939
            || to_year > 2099
            || (to_year + 1) % 20 != 0
            || from_year > to_year
            || to_year - from_year != 19
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
    use super::ClimateApiClient;
    use crate::error::Error;

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

        if let Err(err) = result {
            if let Error::DateRangeNotSupported(1985, 1995) = err {
                ()
            } else {
                panic!("The function returned a wrong error: {}", err.to_string());
            }
        } else {
            panic!("The function call should return an error");
        }
    }

    #[test]
    fn test_averageRainfallForMiddleEarthFrom1980to1999DoesNotExist() {
        let climateApi = ClimateApiClient::new();
        let result = climateApi.get_average_annual_rainfall(1980, 1999, "mde");

        if let Err(err) = result {
            if let Error::NotRecognizedByClimateWeb = err {
                ()
            } else {
                panic!("The function returned a wrong error: {}", err.to_string());
            }
        } else {
            panic!("The function call should return an error");
        }
    }
}
