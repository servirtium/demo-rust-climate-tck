use std::{fmt::Display, io};

#[derive(Debug)]
pub enum Error {
    DateRangeNotSupported(u16, u16),
    NotRecognizedByClimateWeb,
    Deserialization(quick_xml::DeError),
    Reqwest(reqwest::Error),
    Io(io::Error),
}

impl std::error::Error for Error {}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Error::Reqwest(e)
    }
}

impl From<quick_xml::DeError> for Error {
    fn from(e: quick_xml::DeError) -> Self {
        Error::Deserialization(e)
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::DateRangeNotSupported(from_date, to_date) => {
                write!(f, "Date range {}-{} not supported", from_date, to_date)
            }
            Error::NotRecognizedByClimateWeb => write!(f, "Not recognized by ClimateWeb"),
            Error::Reqwest(e) => write!(f, "{}", e),
            Error::Deserialization(e) => write!(f, "{}", e),
            Error::Io(e) => write!(f, "{}", e),
        }
    }
}
