use errors::*;

use std::sync::Arc;

use slog::Logger;
use iron::{Handler, Request, IronResult, IronError, Response};

use dns::DnsService;

struct HttpHandler<Service> {
    logger: Arc<Logger>,
    service: Service,
}

impl<Service: DnsService> HttpHandler<Service> {
    fn new(service: Service, logger: Arc<Logger>) -> Self {
        HttpHandler {
            logger: logger,
            service: service,
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

pub fn run_server<Service>(logger: &Logger, service: Service) -> Result<()>
    where Service: DnsService + Send + Sync + 'static {
    use iron::Iron;

    let logger = Arc::new(logger.new(o!("component" => "iron-server")));
    let handler = HttpHandler::new(service, logger);

    Iron::new(handler)
        .http("127.0.0.1:3000")
        .chain_err(|| "Error during execution of http server")?;
    Ok(())
}
