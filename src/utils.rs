/* Utility functions */
use regex::Regex;
use std::path::Path;

pub fn get_status_text(code: u16) -> &'static str {
    match code {
        // 1xx
        100 => "Continue",
        101 => "Switching Protocols",
        102 => "Processing",
        103 => "Early Hints",
        // 2xx
        200 => "OK",
        201 => "Created",
        202 => "Accepted",
        203 => "Non-Authoritative Information",
        204 => "No Content",
        205 => "Reset Content",
        206 => "Partial Content",
        207 => "Multi-Status",
        208 => "Already Reported",
        226 => "IM Used",
        // 3xx
        300 => "Multiple Choices",
        301 => "Moved Permanently",
        302 => "Found",
        303 => "See Other",
        304 => "Not Modified",
        307 => "Temporary Redirect",
        308 => "Permanent Redirect",
        // 4xx
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        406 => "Not Acceptable",
        407 => "Proxy Authentication Required",
        408 => "Request Timeout",
        409 => "Conflict",
        410 => "Gone",
        411 => "Length Required",
        412 => "Precondition Failed",
        413 => "Payload Too Large",
        414 => "URI Too Long",
        415 => "Unsupported Media Type",
        416 => "Range Not Satisfiable",
        417 => "Expectation Failed",
        418 => "I'm a teapot",
        421 => "Misdirected Request",
        422 => "Unprocessable Entity",
        423 => "Locked",
        424 => "Failed Dependency",
        425 => "Too Early",
        426 => "Upgrade Required",
        428 => "Precondition Required",
        429 => "Too Many Requests",
        431 => "Request Header Fields Too Large",
        451 => "Unavailable For Legal Reasons",
        // 5xx
        500 => "Internal Server Error",
        501 => "Not Implemented",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        504 => "Gateway Timeout",
        505 => "HTTP Version Not Supported",
        506 => "Variant Also Negotiates",
        507 => "Insufficient Storage",
        508 => "Loop Detected",
        510 => "Not Extended",
        511 => "Network Authentication Required",
        _ => "Unknown",
    }
}

pub fn to_titlecase(s: &str) -> String {
    if s.is_empty() {
        String::new()
    } else {
        s[0..1].to_uppercase() + &s[1..].to_lowercase()
    }
}

pub fn sanitize_header_key(key: &str) -> String {
    let re = Regex::new(r"[^a-zA-Z0-9]+").unwrap();

    let result = re.replace_all(key, "-").to_string();
    result
        .split("-")
        .map(|s| to_titlecase(s))
        .collect::<Vec<String>>()
        .join("-")
        .trim_end_matches("-")
        .to_string()
}

pub fn join_path<'a>(prefix: &'a str, path: &'a str) -> String {
    Path::new(prefix)
        .join(path.trim_start_matches('/'))
        .to_string_lossy()
        .to_string()
}

pub fn split_path_query(path: &str) -> (&str, &str) {
    path.split_once('?').unwrap_or((path, ""))
}

pub fn standardize(s: &str) -> String {
    let re = Regex::new(r"[^a-zA-Z0-9-]+").unwrap();
    let res = re.replace_all(s, "-").trim().to_lowercase();
    res.split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<&str>>()
        .join("-")
        .trim_end_matches('-')
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_header_key() {
        assert_eq!(sanitize_header_key("Content-Type"), "Content-Type");
        assert_eq!(sanitize_header_key("Content_Type"), "Content-Type");
        assert_eq!(sanitize_header_key("content-type"), "Content-Type");
        assert_eq!(sanitize_header_key("content_type"), "Content-Type");
        assert_eq!(sanitize_header_key("content_type_"), "Content-Type");
        assert_eq!(sanitize_header_key("content_type_1"), "Content-Type-1");
        assert_eq!(sanitize_header_key("content  type"), "Content-Type");
        assert_eq!(sanitize_header_key("access $^&^&#$& TOKEN"), "Access-Token");
    }

    #[test]
    fn test_join_path() {
        assert_eq!(join_path("/api", "/v1/users"), "/api/v1/users");
        assert_eq!(
            join_path("/api", &join_path("/v1", "/users")),
            "/api/v1/users"
        );
        assert_eq!(join_path("/api", "v1/users"), "/api/v1/users"); // path without leading slash
        assert_eq!(join_path("/api", "v1/users/"), "/api/v1/users/"); // path with trailing slash
        assert_eq!(join_path("api", "v1/users/"), "api/v1/users/");
        assert_eq!(join_path("/", "/v1/users"), "/v1/users");
        assert_eq!(join_path("", "v1/users"), "v1/users");
    }

    #[test]
    fn test_split_path_query() {
        assert_eq!(split_path_query("/users"), ("/users", ""));
        assert_eq!(split_path_query("/users?limit=10"), ("/users", "limit=10"));
        assert_eq!(
            split_path_query("/users?limit=10&offset=0"),
            ("/users", "limit=10&offset=0")
        );
        assert_eq!(
            split_path_query("/users?limit=10&offset=0&sort=name"),
            ("/users", "limit=10&offset=0&sort=name")
        );
        assert_eq!(
            split_path_query("/users?limit=10&offset=0&sort=name&order=asc"),
            ("/users", "limit=10&offset=0&sort=name&order=asc")
        );
        assert_eq!(
            split_path_query("/users/123?limit=10&offset=0&sort=name&order=asc&"),
            ("/users/123", "limit=10&offset=0&sort=name&order=asc&")
        );
    }
}
