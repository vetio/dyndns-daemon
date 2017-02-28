use errors::*;

use std::sync::Arc;

use slog::Logger;
//use iron::{Handler, Request, IronResult, IronError, Response};
use hyper::{header, server};
use hyper::status::StatusCode;

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

    fn authenticate(&self, r: &server::Request) -> bool {
        match r.headers.get::<header::Authorization<header::Basic>>() {
            Some(ref scheme) => {
                let password = match scheme.password {
                    Some(ref p) => p,
                    None => return false,
                };

                if !compare_secure(&scheme.username, &self.username)
                    || !compare_secure(password, &self.password) {
                    false
                } else {
                    true
                }
            },
            None => false,
        }
    }
}

#[inline(never)]
fn compare_secure(s1: &str, s2: &str) -> bool {
    use consistenttime::ct_u8_slice_eq;
    ct_u8_slice_eq(s1.as_bytes(), s2.as_bytes())
}

impl<Service> server::Handler for HttpHandler<Service>
where Service: DnsService + Send + Sync + 'static {
    fn handle(&self, req: server::Request, mut res: server::Response) {

        let logger = self.logger.new(
            o!(
                "url" => format!("{}", req.uri)
            )
        );
        debug!(logger, "{}", req.headers);

        if !self.authenticate(&req) {
            {
                use hyper::status::StatusCode;
                let status = res.status_mut();
                *status = StatusCode::Unauthorized;
            }
            {
                let header = res.headers_mut();
                header.set_raw("WWW-Authenticate", vec![b"Basic".to_vec()]);
            }
            return;
        }

        use std::str::FromStr;
        use std::net::Ipv4Addr;

        match req.headers.get_raw(&self.addr_header) {
            Some(values) => {
                let result = values.first()
                    .ok_or_else(|| "No values for ip header".into())
                    .and_then(
                        |val| String::from_utf8(val.clone())
                            .chain_err(|| "Error reading header as utf-8")
                    )
                    .and_then(
                        |s| Ipv4Addr::from_str(&s)
                            .chain_err(|| "Error interpreting address as ipv4")
                    )
                    .and_then(|ip| self.service.update(&ip));
                if let Err(e) = result {
                    log_error(&self.logger, &e);

                    let status = res.status_mut();
                    *status = StatusCode::InternalServerError;
                    return;
                }
            },
            None => {
                let status = res.status_mut();
                *status = StatusCode::BadRequest;
                return;
            }
        };

        let status = res.status_mut();
        *status = StatusCode::Ok;
    }
}

pub fn run_server<Service>(logger: &Logger, service: Service, config: &config::Config) -> Result<()>
    where Service: DnsService + Send + Sync + 'static {
    use hyper::server::Server;

    let logger = Arc::new(logger.new(o!("component" => "iron-server")));
    let handler = HttpHandler::new(service, logger, config);

    Server::http(&config.server_addr)
        .chain_err(|| "Error starting http server")?
        .handle(handler)
        .chain_err(|| "Error during handler execution")?;
    Ok(())
}
