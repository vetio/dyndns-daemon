use errors::*;

use std::sync::Arc;

use slog::Logger;
use iron::{Handler, Request, IronResult, IronError, Response};

use dns::DnsService;
use config;

struct HttpHandler<Service> {
    logger: Arc<Logger>,
    service: Service,
    username: String,
    password: String,
}

impl<Service: DnsService> HttpHandler<Service> {
    fn new(service: Service, logger: Arc<Logger>, config: &config::Config) -> Self {
        HttpHandler {
            logger: logger,
            service: service,
            username: config.http_auth_user.clone(),
            password: config.http_auth_password.clone(),
        }
    }

    fn generate_auth_error(&self) -> Response {
        use iron::status;
        let mut response = Response::with(status::Unauthorized);

        response.headers.set_raw("WWW-Authenticate", vec![b"Basic".to_vec()]);

        response
    }

    fn authenticate(&self, r: &mut Request) -> Option<Response> {
        use iron::headers;

        match r.headers.get::<headers::Authorization<headers::Basic>>() {
            Some(ref scheme) => {
                if scheme.username != self.username || scheme.password.as_ref() != Some(&self.password) {
                    Some(self.generate_auth_error())
                } else {
                    None
                }
            },
            None => Some(self.generate_auth_error()),
        }
    }
}

impl<Service> Handler for HttpHandler<Service>
where Service: DnsService + Send + Sync + 'static {
    fn handle(&self, r: &mut Request) -> IronResult<Response> {
        use iron::status;

        let logger = self.logger.new(
            o!(
                "url" => format!("{}", r.url)
            )
        );
        debug!(logger, "{}", r.headers);

        if let Some(error) = self.authenticate(r) {
            return Ok(error);
        }

        use std::net::SocketAddr;

        match r.remote_addr {
            SocketAddr::V4(ref addr) => {
                if let Err(e) = self.service.update(addr.ip()) {
                    log_error(&self.logger, &e);
                    return Err(IronError::new(
                        e,
                        status::InternalServerError
                    ));
                }
            },
            SocketAddr::V6(ref addr) => {
                info!(logger, "request from ipv6 address {}", addr);
                return Ok(Response::with(status::Ok));
            }
        };

        Ok(Response::with(status::Ok))
    }
}

pub fn run_server<Service>(logger: &Logger, service: Service, config: &config::Config) -> Result<()>
    where Service: DnsService + Send + Sync + 'static {
    use iron::Iron;

    let logger = Arc::new(logger.new(o!("component" => "iron-server")));
    let handler = HttpHandler::new(service, logger, config);

    Iron::new(handler)
        .http(&config.server_addr)
        .chain_err(|| "Error during execution of http server")?;
    Ok(())
}
