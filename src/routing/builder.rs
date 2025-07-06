use crate::http::HttpMethod;
use crate::routing::route::RouteHandler;

#[derive(Debug, thiserror::Error)]
pub enum RouteBuilderError {
    #[error("Path cannot be empty")]
    EmptyPath,

    #[error("Path must start with '/'")]
    InvalidPath(String),

    #[error("Route already exists: {method:?} {path}")]
    DuplicateRoute { method: HttpMethod, path: String },

    #[error("Invalid parameter name: {0}")]
    InvalidParameter(String),

    #[error("Handler is required")]
    MissingHandler,
}

pub trait RouteBuilder {
    type Error;

    fn register(&mut self, path: &str, method: HttpMethod, handler: RouteHandler);

    fn get(&mut self, path: &str, handler: RouteHandler) {
        self.register(path, HttpMethod::GET, handler);
    }
    fn post(&mut self, path: &str, handler: RouteHandler) {
        self.register(path, HttpMethod::POST, handler);
    }
    fn put(&mut self, path: &str, handler: RouteHandler) {
        self.register(path, HttpMethod::PUT, handler);
    }
    fn patch(&mut self, path: &str, handler: RouteHandler) {
        self.register(path, HttpMethod::PATCH, handler);
    }
    fn delete(&mut self, path: &str, handler: RouteHandler) {
        self.register(path, HttpMethod::DELETE, handler);
    }
    fn options(&mut self, path: &str, handler: RouteHandler) {
        self.register(path, HttpMethod::OPTIONS, handler);
    }
    fn head(&mut self, path: &str, handler: RouteHandler) {
        self.register(path, HttpMethod::HEAD, handler);
    }
    fn add_route(&mut self, method: HttpMethod, path: &str, handler: RouteHandler) {
        self.register(path, method, handler);
    }
}

pub fn validate_route_path(path: &str) -> Result<(), RouteBuilderError> {
    if path.is_empty() {
        return Err(RouteBuilderError::EmptyPath);
    }

    if !path.starts_with('/') {
        return Err(RouteBuilderError::InvalidPath(
            "Path must start with '/'".to_string(),
        ));
    }

    // Check for invalid characters
    if path.contains("//") {
        return Err(RouteBuilderError::InvalidPath(
            "Path cannot contain double slashes".to_string(),
        ));
    }

    Ok(())
}

pub fn normalize_path(path: &str) -> String {
    let path = path.trim_start_matches('/');
    if path.is_empty() {
        "/".to_string()
    } else {
        format!("/{}", path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_route_path() {
        assert!(validate_route_path("/users").is_ok());
        assert!(validate_route_path("/users/123").is_ok());
        assert!(validate_route_path("/users/:id").is_ok());
        assert!(validate_route_path("/users/:id/messages/:message_id").is_ok());
        assert!(validate_route_path("//users/:id/messages/:message_id").is_err());
        assert!(validate_route_path("users/123/messages/456").is_err());
    }
}
