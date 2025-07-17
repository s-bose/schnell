use crate::utils::standardize;
use std::collections::HashMap;

pub fn parse_headers<'a>(lines: &'a [String]) -> HashMap<String, &'a str> {
    let mut headers = HashMap::new();
    for line in lines {
        if let Some((key, value)) = line.split_once(':') {
            headers.insert(standardize(key), value.trim());
        }
    }

    headers
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_headers() {
        let lines = vec![
            "Content-Type: text/html".to_string(),
            "Content-Length: 123".to_string(),
            "Set-Cookie: sessionId=abc123".to_string(),
        ];
        let headers = parse_headers(&lines);
        assert_eq!(headers.get("content-type"), Some(&"text/html"));
        assert_eq!(headers.get("content-length"), Some(&"123"));
        assert_eq!(headers.get("set-cookie"), Some(&"sessionId=abc123"));
    }
}
