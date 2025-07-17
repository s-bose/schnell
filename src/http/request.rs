use crate::http::{HttpMethod, Version, method};
use crate::utils::split_path_query;
use std::default::Default;
use std::{
    collections::HashMap,
    io::{BufRead, BufReader, ErrorKind, Read},
    str::FromStr,
};

pub enum RequestError {
    ReadError,
    InvalidRequest,
    RequestTooLarge,
    ConnectionClosed,
    ConnectionTimedOut,
    ParseError,
}

#[derive(Debug)]
pub struct Request {
    pub method: HttpMethod,
    pub path: String,
    pub version: Version,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub path_params: HashMap<String, String>,
    pub query_params: HashMap<String, String>,
}

impl Default for Request {
    fn default() -> Self {
        Self {
            method: HttpMethod::GET,
            path: String::new(),
            version: Version::HTTP1_1,
            headers: HashMap::new(),
            body: String::new(),
            path_params: HashMap::new(),
            query_params: HashMap::new(),
        }
    }
}

impl Request {
    pub fn read<R: Read>(mut buffer: BufReader<R>) -> Result<Self, RequestError> {
        let mut lines = Vec::new();
        let mut line = String::new();

        loop {
            match buffer.read_line(&mut line) {
                Ok(0) => {
                    // End of stream reached
                    if lines.is_empty() {
                        return Err(RequestError::ConnectionClosed);
                    }
                    break;
                }
                Ok(_) => {
                    if line.trim().is_empty() {
                        break; // End of headers
                    }
                    lines.push(line.trim().to_string());
                    line.clear();
                }
                Err(e) => match e.kind() {
                    std::io::ErrorKind::UnexpectedEof => {
                        return Err(RequestError::ConnectionClosed);
                    }
                    std::io::ErrorKind::TimedOut => {
                        return Err(RequestError::ConnectionTimedOut);
                    }
                    std::io::ErrorKind::ConnectionReset
                    | std::io::ErrorKind::ConnectionAborted
                    | std::io::ErrorKind::BrokenPipe => {
                        return Err(RequestError::ConnectionClosed);
                    }
                    _ => {
                        return Err(RequestError::ReadError);
                    }
                },
            }
        }

        if lines.is_empty() {
            return Err(RequestError::ConnectionClosed);
        }

        if buffer.buffer().len() > 1024 * 1024 * 10 {
            return Err(RequestError::RequestTooLarge);
        }

        // Parse request line
        let (method, path, version) = Self::parse_request_line(&lines[0])?;

        // Parse headers
        let headers = Self::parse_headers(&lines[1..]);

        // Parse body (read remaining content)
        let body = Self::parse_body(&mut buffer, &headers)?;

        let (path_seg, _) = split_path_query(&path);

        Ok(Request {
            method,
            path: String::from(path_seg),
            version,
            headers,
            body,
            path_params: HashMap::new(),
            query_params: HashMap::new(),
        })
    }

    pub fn query(&self, key: &str) -> Option<&str> {
        self.query_params.get(key).map(|v| v.as_str())
    }

    pub fn path(&self, key: &str) -> Option<&str> {
        self.path_params.get(key).map(|v| v.as_str())
    }

    pub fn add_path_params(&mut self, path_params: HashMap<String, String>) {
        self.path_params.extend(path_params);
    }

    pub fn add_query_params(&mut self, query_params: HashMap<String, String>) {
        self.query_params.extend(query_params);
    }

    fn parse_request_line(line: &str) -> Result<(HttpMethod, &str, Version), RequestError> {
        let mut parts = line.split_whitespace();

        let method_str = parts.next().ok_or(RequestError::ParseError)?;
        let path = parts.next().ok_or(RequestError::ParseError)?;
        let version_str = parts.next().ok_or(RequestError::ParseError)?;

        if parts.next().is_some() {
            return Err(RequestError::ParseError);
        }

        Ok((
            HttpMethod::from_str(method_str).map_err(|_| RequestError::InvalidRequest)?,
            path,
            Version::from_str(version_str).map_err(|_| RequestError::InvalidRequest)?,
        ))
    }

    fn parse_headers(lines: &[String]) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        for line in lines {
            if let Some((key, value)) = line.split_once(':') {
                headers.insert(key.trim().to_lowercase(), value.trim().to_string());
            }
        }
        headers
    }

    fn parse_body<R: Read>(
        buffer: &mut BufReader<R>,
        headers: &HashMap<String, String>,
    ) -> Result<String, RequestError> {
        let content_length = headers
            .get("content-length")
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(0);

        if content_length == 0 {
            return Ok(String::new());
        }

        let mut body = vec![0; content_length];
        match buffer.read_exact(&mut body) {
            Ok(()) => {}
            Err(e) => match e.kind() {
                ErrorKind::UnexpectedEof => {
                    return Err(RequestError::ConnectionClosed);
                }
                ErrorKind::TimedOut => {
                    return Err(RequestError::ConnectionTimedOut);
                }
                ErrorKind::ConnectionReset
                | ErrorKind::ConnectionAborted
                | ErrorKind::BrokenPipe => {
                    return Err(RequestError::ConnectionClosed);
                }
                _ => {
                    return Err(RequestError::ReadError);
                }
            },
        }

        String::from_utf8(body).map_err(|_| RequestError::ParseError)
    }
}
