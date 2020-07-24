use io::{BufRead, BufReader, Write};
use lazy_static::lazy_static;
use regex::Regex;
use std::{
    fmt::Display,
    fs, io,
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    sync::{self, Mutex, MutexGuard},
    thread,
};
use sync::Once;

lazy_static! {
    static ref HEADER_REGEX: Regex =
        Regex::new(r"(?m)(?P<header_key>[a-zA-Z\-]+): (?P<header_value>.*?)$").unwrap();

    static ref MARKDOWN_REGEX: Regex = Regex::new(
            "(?ms)\\#\\# [^/]*(?P<uri>.*\\.xml).*?\\#\\#\\# Response headers recorded for playback.*?```\
            \\s*(?P<headers_part>.*?)\\s*```.*?\\#\\#\\# Response body recorded for playback.*?```\\s*\
            (?P<body_part>.*?)\\s*```.*?")
        .unwrap();

    static ref TEST_LOCK: Mutex<()> = Mutex::new(());
}

pub enum ServirtiumMode {
    Playback,
    Record,
}

static SERVIRTIUM_INIT: Once = Once::new();

lazy_static! {
    static ref SERVIRTIUM_INSTANCE: Mutex<ServirtiumServer> = Mutex::new(ServirtiumServer::new());
}

pub struct ServirtiumServer {
    interaction_mode: ServirtiumInteractionMode,
    domain_name: Option<String>,
}

#[derive(Debug, Clone)]
enum ServirtiumInteractionMode {
    Playback(PlaybackData),
    Recording(PathBuf),
    NotSet,
}

impl ServirtiumServer {
    fn new() -> Self {
        ServirtiumServer {
            interaction_mode: ServirtiumInteractionMode::NotSet,
            domain_name: None,
        }
    }

    pub fn prepare_for_test<P: AsRef<Path>, S: Into<String>>(
        mode: ServirtiumMode,
        script_path: P,
        domain_name: S,
    ) -> Result<MutexGuard<'static, ()>, ServirtiumServerError> {
        Self::start();

        let test_lock = TEST_LOCK.lock()?;

        let mut server_lock = SERVIRTIUM_INSTANCE.lock()?;
        server_lock.domain_name = Some(domain_name.into());
        server_lock.interaction_mode = match mode {
            ServirtiumMode::Playback => {
                let playback_data = Self::load_playback_file(script_path)?;
                ServirtiumInteractionMode::Playback(playback_data)
            }
            ServirtiumMode::Record => {
                ServirtiumInteractionMode::Recording(PathBuf::from(script_path.as_ref()))
            }
        };

        Ok(test_lock)
    }

    fn start() {
        SERVIRTIUM_INIT.call_once(|| {
            thread::spawn(|| {
                let listener = TcpListener::bind("localhost:61417").unwrap();

                for stream in listener.incoming() {
                    if let Ok(mut stream) = stream {
                        let servirtium_instance = SERVIRTIUM_INSTANCE.lock().unwrap();
                        match &servirtium_instance.interaction_mode {
                            ServirtiumInteractionMode::Playback(playback_data) => {
                                Self::handle_playback(&mut stream, &playback_data);
                            }
                            ServirtiumInteractionMode::Recording(path) => {
                                Self::handle_record(&mut stream, path);
                            }
                            ServirtiumInteractionMode::NotSet => {}
                        };
                    }
                }
            });
        });
    }

    fn handle_playback(stream: &mut TcpStream, playback_data: &PlaybackData) {
        // it's necessary because the client first needs to send all the data it needs to send
        let _ = Self::read_first_line(stream);

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

    fn handle_record<P: AsRef<Path>>(_stream: &mut TcpStream, _record_path: P) {
        todo!()
    }

    fn load_playback_file<P: AsRef<Path>>(
        filename: P,
    ) -> Result<PlaybackData, ServirtiumServerError> {
        let file_contents = fs::read_to_string(filename)?;

        let markdown_captures = MARKDOWN_REGEX
            .captures(&file_contents)
            .ok_or(ServirtiumServerError::InvalidMarkdownFormat)?;

        let uri = &markdown_captures["uri"];
        let headers_part = &markdown_captures["headers_part"];
        let body_part = &markdown_captures["body_part"];

        let headers = Self::parse_headers(headers_part);

        Ok(PlaybackData {
            headers,
            response_body: String::from(body_part),
            uri: String::from(uri),
        })
    }

    fn parse_headers<T: AsRef<str>>(headers_part: T) -> Vec<(String, String)> {
        let mut headers = Vec::new();

        for capture in HEADER_REGEX.captures_iter(headers_part.as_ref()) {
            headers.push((
                String::from(capture["header_key"].trim()),
                String::from(capture["header_value"].trim()),
            ));
        }

        headers
    }

    fn read_first_line(stream: &mut TcpStream) -> Result<String, io::Error> {
        let mut tmp_str = String::new();
        let mut reader = BufReader::new(&*stream);
        reader.read_line(&mut tmp_str).map(|_| tmp_str)
    }
}

impl Default for ServirtiumServer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub enum ServirtiumServerError {
    InvalidMarkdownFormat,
    IoError(io::Error),
    PoisonedLock,
}

impl std::error::Error for ServirtiumServerError {}

impl Display for ServirtiumServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServirtiumServerError::InvalidMarkdownFormat => {
                write!(f, "The markdown format was poisoned")
            }
            ServirtiumServerError::IoError(e) => write!(f, "IoError: {}", e.to_string()),
            ServirtiumServerError::PoisonedLock => write!(f, "The lock was poisoned"),
        }
    }
}

impl From<io::Error> for ServirtiumServerError {
    fn from(e: io::Error) -> Self {
        ServirtiumServerError::IoError(e)
    }
}

impl<T> From<sync::PoisonError<T>> for ServirtiumServerError {
    fn from(_: sync::PoisonError<T>) -> Self {
        ServirtiumServerError::PoisonedLock
    }
}

#[derive(Debug, Clone)]
struct PlaybackData {
    pub uri: String,
    pub headers: Vec<(String, String)>,
    pub response_body: String,
}
