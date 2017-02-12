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
    addr_header: String,
}

impl<Service: DnsService> HttpHandler<Service> {
    fn new(service: Service, logger: Arc<Logger>, config: &config::Config) -> Self {
        HttpHandler {
            logger: logger,
            service: service,
            username: config.http_auth_user.clone(),
            password: config.http_auth_password.clone(),
            addr_header: config.ip_header.clone(),
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

        use std::str::FromStr;
        use std::net::Ipv4Addr;

        match r.headers.get_raw(&self.addr_header) {
            Some(values) => {
                if let Err(e) = values.first().ok_or_else(|| "No values for ip header".into())
                    .and_then(
                        |val| String::from_utf8(val.clone())
                            .chain_err(|| "Error reading header as utf-8")
                    )
                    .and_then(
                        |s| Ipv4Addr::from_str(&s)
                            .chain_err(|| "Error interpreting address as ipv4")
                    )
                    .and_then(|ip| self.service.update(&ip)) {
                    log_error(&self.logger, &e);
                    return Err(IronError::new(
                        e,
                        status::InternalServerError
                    ));
                }
            },
            None => {
                return Ok(Response::with(status::BadRequest));
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
