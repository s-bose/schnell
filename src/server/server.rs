use crate::routing::route::{Route, RouteError, register_route};
use crate::routing::{builder::RouteBuilder, route::resolve_route};
use crate::{
    http::{
        HttpMethod, HttpResponse,
        request::{Request, RequestError},
        response,
    },
    utils::split_path_query,
};
use scoped_threadpool::Pool;
use std::default::Default;
use std::io::BufReader;
use std::{
    net::{TcpListener, TcpStream},
    time::Duration,
};

use crate::routing::router::RouteGroup;
use crate::utils::join_path;

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pool_size: Option<usize>,
    read_timeout_ms: Option<Duration>,
    write_timeout_ms: Option<Duration>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            pool_size: None,
            read_timeout_ms: None,
            write_timeout_ms: None,
        }
    }
}

#[derive(Debug)]
pub struct Server {
    ip_addr: String,
    port: u16,
    routes: Vec<Route>,
    config: ServerConfig,
}

impl RouteBuilder for Server {
    type Error = RouteError;

    fn register(
        &mut self,
        path: &str,
        method: HttpMethod,
        handler: crate::routing::route::RouteHandler,
    ) {
        let (stripped_path, _) = split_path_query(&path);
        register_route(&mut self.routes, "/", stripped_path, method, handler);
    }
}

impl Server {
    pub fn new(ip_addr: &str, port: u16) -> Self {
        Self {
            ip_addr: ip_addr.to_owned(),
            port,
            routes: Vec::new(),
            config: ServerConfig::default(),
        }
    }

    pub fn with_config(self, config: ServerConfig) -> Self {
        Self { config, ..self }
    }

    pub fn group<F>(&mut self, prefix: &str, config: F)
    where
        F: FnOnce(&mut RouteGroup),
    {
        let mut group = RouteGroup {
            prefix: join_path("/", &split_path_query(prefix).0),
            routes: &mut self.routes,
        };

        config(&mut group);
    }

    pub fn listen(&self) -> ! {
        let listener = TcpListener::bind(format!("{}:{}", self.ip_addr, self.port))
            .expect("Error starting server");

        log::info!("Server listening on {}:{}", self.ip_addr, self.port);

        self.listen_with_pool(self.config.pool_size, listener);
    }

    pub fn listen_with_pool(&self, pool_size: Option<usize>, listener: TcpListener) -> ! {
        let logical_cores = num_cpus::get() as u32;
        let pool_size = pool_size.unwrap_or(logical_cores as usize);

        let mut pool = Pool::new(pool_size as u32);

        let mut incoming = listener.incoming();

        loop {
            let mut stream = incoming
                .next()
                .unwrap()
                .expect("Error accepting TCP connection");

            if let Err(e) = stream.set_read_timeout(self.config.read_timeout_ms) {
                log::error!("Error setting read timeout: {:?}", e);
                self.send_response(&mut stream, HttpResponse::internal_server_error());
            }

            if let Err(e) = stream.set_write_timeout(self.config.write_timeout_ms) {
                log::error!("Error setting write timeout: {:?}", e);
                self.send_response(&mut stream, HttpResponse::internal_server_error());
            }

            pool.scoped(|scope| {
                scope.execute(|| {
                    self.handle_connection(stream);
                });
            })
        }
    }

    pub fn handle_connection(&self, mut stream: TcpStream) {
        let mut request = match Request::read(BufReader::new(&mut stream)) {
            Err(
                RequestError::ReadError | RequestError::ParseError | RequestError::InvalidRequest,
            ) => {
                log::error!("Error reading request");
                self.send_response(&mut stream, HttpResponse::internal_server_error());
                return;
            }
            Err(RequestError::RequestTooLarge) => {
                log::error!("Request too large");
                self.send_response(&mut stream, HttpResponse::request_entity_too_large());
                return;
            }
            Err(RequestError::ConnectionClosed) => {
                log::info!("Client connection closed");
                return;
            }
            Err(RequestError::ConnectionTimedOut) => {
                log::error!("Client connection timed out");
                return;
            }
            Ok(request) => request,
        };

        let (route, rm) = match resolve_route(&self.routes, &request.path, request.method.clone()) {
            Ok((route, rm)) => (route, rm),
            Err(RouteError::MethodNotAllowed) => {
                self.send_response(&mut stream, HttpResponse::method_not_allowed());
                return;
            }
            Err(RouteError::NotFound) => {
                self.send_response(&mut stream, HttpResponse::not_found());
                return;
            }
        };

        request.add_path_params(rm.path_params);
        request.add_query_params(rm.query_params);

        let response = (route.handler)(&request);

        match response {
            Ok(response) => {
                self.send_response(&mut stream, response);
            }
            Err(err) => {
                log::error!("Error writing response: {:?}", err);
                self.send_response(&mut stream, HttpResponse::internal_server_error());
            }
        }
    }

    fn send_response(&self, stream: &mut TcpStream, response: HttpResponse) {
        if let Err(err) = response::write_response(stream, response) {
            log::error!("Error writing response: {:?}", err);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::http::response::HttpResponse;

    #[test]
    fn test_server_group_route() {
        let mut server = Server::new("127.0.0.1", 8080);
        server.group("/api", |group| {
            group.get("/users", |_| Ok(HttpResponse::ok()));
            group.post("/users", |_| Ok(HttpResponse::ok()));
            group.get("/users/:id", |_| Ok(HttpResponse::ok()));
            group.get("/users/:id/messages?limit=10", |_| Ok(HttpResponse::ok()));
        });

        server.get("/debug/telemetry", |_| Ok(HttpResponse::ok()));

        assert_eq!(server.routes[0].path, "/api/users");
        assert_eq!(server.routes[1].path, "/api/users");
        assert_eq!(server.routes[2].path, "/api/users/:id");
        assert_eq!(server.routes[3].path, "/api/users/:id/messages");
        assert_eq!(server.routes[4].path, "/debug/telemetry");
    }
}
