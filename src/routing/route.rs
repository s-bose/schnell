use crate::http::{HttpMethod, HttpResponse, Request};
use crate::utils::{join_path, split_path_query};

use std::collections::HashMap;

pub type RouteHandler = fn(&Request) -> std::io::Result<HttpResponse>;
#[derive(Debug, PartialEq)]
pub struct Route {
    pub method: HttpMethod,
    pub path: String,
    pub handler: RouteHandler,
}

#[derive(Debug, PartialEq)]
pub enum RouteError {
    NotFound,
    MethodNotAllowed,
}

#[derive(Debug, PartialEq)]
pub struct RouteMatch {
    pub path: String,
    pub path_params: HashMap<String, String>,
    pub query_params: HashMap<String, String>,
}

pub fn parse_query(query: &str) -> HashMap<String, String> {
    if query.is_empty() {
        return HashMap::new();
    }
    let mut query_map = HashMap::new();
    for pair in query.split('&') {
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        query_map.insert(key.to_string(), value.to_string());
    }
    query_map
}

pub fn parse_path(path: &str, incoming: &str) -> HashMap<String, String> {
    if incoming.is_empty() {
        return HashMap::new();
    }
    let mut path_map = HashMap::new();
    for (route_part, incoming_part) in path.split('/').zip(incoming.split('/')) {
        if route_part.starts_with(':') {
            path_map.insert(
                route_part.trim_start_matches(':').to_string(),
                incoming_part.to_string(),
            );
        }
    }
    path_map
}

pub fn match_route<'a>(route: &str, incoming: &str) -> Option<RouteMatch> {
    let (route_path, _) = split_path_query(route);
    let (incoming_path, incoming_query) = split_path_query(incoming);

    let route_parts = route_path.split('/');
    let incoming_parts = incoming_path.split('/');

    if route_parts.count() != incoming_parts.count() {
        return None;
    }

    Some(RouteMatch {
        path: route_path.to_string(),
        path_params: parse_path(route_path, incoming_path),
        query_params: parse_query(incoming_query),
    })
}

pub fn register_route(
    routes: &mut Vec<Route>,
    prefix: &str,
    path: &str,
    method: HttpMethod,
    handler: RouteHandler,
) {
    let path = join_path(&prefix, path);
    if let Some(matching_route_idx) = routes
        .iter()
        .position(|r| r.path == path && r.method == method)
    {
        log::warn!(
            "Route {:?} {:?} already exists and will be overwritten",
            method,
            path
        );
        routes.insert(
            matching_route_idx,
            Route {
                path,
                method,
                handler,
            },
        );
    } else {
        routes.push(Route {
            path,
            method,
            handler,
        });
    }
}

pub fn resolve_route<'a>(
    routes: &'a Vec<Route>,
    path: &str,
    method: HttpMethod,
) -> Result<(&'a Route, RouteMatch), RouteError> {
    for route in routes {
        if let Some(m) = match_route(&route.path, path) {
            if route.method == method {
                return Ok((route, m));
            }
            return Err(RouteError::MethodNotAllowed);
        }
    }

    Err(RouteError::NotFound)
}

#[cfg(test)]
mod tests {
    use crate::routing::builder::RouteBuilder;

    use super::*;

    #[test]
    fn test_match_route() {
        assert!(match_route("/", "/").is_some());
        assert!(match_route("/users", "/users").is_some());
        assert!(match_route("/users/:id", "/users/123").is_some());
        assert_eq!(
            match_route("/users/messages/:message_id", "/users/:userid"),
            None
        );
        assert_eq!(
            match_route(
                "/users/:user_id/messages/:message_id",
                "/users/123/messages/456"
            ),
            Some(RouteMatch {
                path: "/users/:user_id/messages/:message_id".to_string(),
                path_params: HashMap::from([
                    ("user_id".to_string(), "123".to_string()),
                    ("message_id".to_string(), "456".to_string()),
                ]),
                query_params: HashMap::new(),
            })
        );
        assert_eq!(
            match_route("/users/messages/:message_id", "/users/123/messages/456/"),
            None
        );
        assert_eq!(
            match_route(
                "/users/:user_id/messages/:message_id",
                "/users/123/messages/456?limit=10"
            ),
            Some(RouteMatch {
                path: "/users/:user_id/messages/:message_id".to_string(),
                path_params: HashMap::from([
                    ("user_id".to_string(), "123".to_string()),
                    ("message_id".to_string(), "456".to_string()),
                ]),
                query_params: HashMap::from([("limit".to_string(), "10".to_string())]),
            })
        );
    }

    // #[test]
    // fn test_route_resolver() {
    //     struct TestRouter {
    //         routes: Vec<Route>,
    //     }

    //     impl RouteBuilder for TestRouter {
    //         type Error = RouteError;

    //         fn register(&mut self, path: &str, method: HttpMethod, handler: RouteHandler) {
    //             self.routes.push(Route {
    //                 method,
    //                 path: path.to_string(),
    //                 handler,
    //             });
    //         }
    //     }

    //     let _routes = vec![
    //         Route {
    //             method: HttpMethod::GET,
    //             path: "/users".to_string(),
    //             handler: |_| Ok(HttpResponse::ok()),
    //         },
    //         Route {
    //             method: HttpMethod::POST,
    //             path: "/users".to_string(),
    //             handler: |_| Ok(HttpResponse::ok()),
    //         },
    //         Route {
    //             method: HttpMethod::GET,
    //             path: "/users/:id".to_string(),
    //             handler: |_| Ok(HttpResponse::ok()),
    //         },
    //         Route {
    //             method: HttpMethod::GET,
    //             path: "/users/:id/messages/:message_id".to_string(),
    //             handler: |_| Ok(HttpResponse::ok()),
    //         },
    //     ];

    //     let mut request = Request {
    //         path: "/users".to_string(),
    //         method: HttpMethod::GET,
    //         ..Default::default()
    //     };
    // }

    //     let route = resolve_route(&routes, &mut request);
    //     assert!(route.is_ok());
    //     assert_eq!(route.unwrap().path, "/users");

    //     let mut request = Request {
    //         path: "/users/123".to_string(),
    //         method: HttpMethod::GET,
    //         ..Default::default()
    //     };

    //     let route = resolve_route(&routes, &mut request);
    //     assert!(route.is_ok());
    //     assert_eq!(route.unwrap().path, "/users/:id");

    //     let mut request = Request {
    //         path: "/users/123/messages/456".to_string(),
    //         method: HttpMethod::GET,
    //         ..Default::default()
    //     };

    //     let route = resolve_route(&routes, &mut request);
    //     assert!(route.is_ok());
    //     assert_eq!(route.unwrap().path, "/users/:id/messages/:message_id");

    //     let mut request = Request {
    //         path: "/users/123/messages/456".to_string(),
    //         method: HttpMethod::POST,
    //         ..Default::default()
    //     };

    //     let route = resolve_route(&routes, &mut request);
    //     assert!(route.is_err());
    //     assert_eq!(route.unwrap_err(), RouteError::MethodNotAllowed);
    // }
}
