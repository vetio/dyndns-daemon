use errors::*;

//use std::net::Ipv4Addr;
use std::sync::Arc;

use slog::Logger;
//use iron::{Handler, Request, IronResult, IronError, Response};
//use hyper::StatusCode;
use hyper::{Request, Response, StatusCode};

use config;
use dns::DnsService;
use std::borrow::Cow;

struct HttpHandler<Service> {
    logger: Arc<Logger>,
    service: Service,
    username: String,
    password: String,
    ip_resolv: config::IpResolvMethod,
}

impl<Service: DnsService> HttpHandler<Service> {
    fn new(service: Service, logger: Arc<Logger>, config: &config::Config) -> Self {
        HttpHandler {
            logger,
            service,
            username: config.http_auth_user.clone(),
            password: config.http_auth_password.clone(),
            ip_resolv: config.ip_resolv.clone(),
        }
    }
}

fn authenticate<R>(config: &config::Config, r: &Request<R>) -> bool {
    match r.headers().get(::hyper::header::AUTHORIZATION) {
        Some(scheme) => {
            let token = match scheme.to_str() {
                Ok(t) => t,
                Err(_) => return false,
            };
            let mut expected = String::from("Basic ");
            expected += &::base64::encode(&format!("{}:{}", &config.http_auth_user,
            &config.http_auth_password));
            compare_secure(&expected, token)
            //            let password = match scheme.password {
            //                Some(ref p) => p,
            //                None => return false,
            //            };
            //
            //            compare_secure(&scheme.username, &config.http_auth_user)
            //                && compare_secure(password, &config.http_auth_password)
        }
        None => false,
    }
}

#[inline(never)]
fn compare_secure(s1: &str, s2: &str) -> bool {
    use consistenttime::ct_u8_slice_eq;
    ct_u8_slice_eq(s1.as_bytes(), s2.as_bytes())
}

fn handle_request<R, Service>(
    req: Request<R>,
    logger: &Logger,
    config: &config::Config,
    service: &Service,
) -> ::http::Result<Response<::hyper::Body>>
where
    Service: DnsService,
{
    let logger = logger.new(o!(
                "url" => format!("{}", req.uri())
            ));
    debug!(logger, "{:?}", req.headers());

    if !authenticate(&config, &req) {
        return Response::builder()
            .status(::hyper::StatusCode::UNAUTHORIZED)
            .header(::hyper::header::WWW_AUTHENTICATE, "Basic")
            .body("".into());
    }

    use std::net::Ipv4Addr;
    use std::str::FromStr;

    let ip = match resolv_ip_from_request(&config.ip_resolv, &req) {
        Some(ip) => ip,
        None => {
            return Response::builder().status(StatusCode::BAD_REQUEST).body("".into());
        }
    };

    let result = ip.and_then(|s| {
        Ipv4Addr::from_str(&s).chain_err(|| "Error interpreting address as ipv4")
    }).and_then(|ip| service.update(&ip));

    if let Err(e) = result {
        log_error(&logger, &e);

        return Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body("".into());
    };

    Response::builder().status(StatusCode::OK).body("".into())
}

fn find_in_query<'a, 'b>(query: &'a str, name: &'b str) -> Option<Cow<'a, str>> {
    use ::url::form_urlencoded::parse;

    parse(query.as_bytes())
        .find(|(key, _)| &*key == name)
        .map(|(_, val)| val)
}

fn resolv_ip_from_request<R>(
    method: &config::IpResolvMethod,
    req: &Request<R>,
) -> Option<Result<String>> {
    match method {
        config::IpResolvMethod::DynDns2 => {
            static DOMAIN_HEADER: &str = "domain";
            static IP_HEADER: &str = "myip";

            let query = req.uri().query()?;
            let domain = find_in_query(query, DOMAIN_HEADER)?;
            let ip = find_in_query(query, IP_HEADER)?;
            Some(Ok(ip.to_string()))
//                .map(|s| s.as_bytes())
//                .map(::url::form_urlencoded::parse)
//                .and_then(|mut i| i.find(|(key, _)| &*key == param_name))
//                .map(|(_, val)| Ok(val.to_string()))
//            let url = format!("{}", req.uri);
//            let url = match ::url::Url::parse(&url) {
//                Ok(url) => url,
//                Err(e) => return Some(Err(e.to_string().into())),
//            };
//
//            match url.query_pairs()
//                .filter(|&(ref key, _)| key == param_name.as_str())
//                .next()
//            {
//                Some((_, val)) => Some(Ok(val.to_string())),
//                None => return None,
//            }
        }
        config::IpResolvMethod::Header(header_name) =>
            match req.headers().get(header_name) {
            Some(value) => {
                Some(
                    value
                        .to_str()
                        .chain_err(|| "Invalid value for ip header")
                        .map(String::from)
                )
            },
            None => None,
        },
    }
}

pub fn run_server<Service>(logger: &Logger, service: Service, config: Arc<config::Config>) -> Result<()>
where
    Service: DnsService + Send + Sync + 'static,
{
    use hyper::server::Server;
    let logger = Arc::new(logger.new(o!("component" => "iron-server")));
    let service = Arc::new(service);
//    let handler = HttpHandler::new(service, logger, config);
    use hyper::rt::Future;

    let addr = config
        .server_addr
        .parse()
        .chain_err(|| "Error parsing server address")?;

    let new_service = move || {
        let logger = logger.clone();
        let config = config.clone();
        let service = service.clone();
        ::hyper::service::service_fn_ok(
            move |req: Request<::hyper::Body>| {
                handle_request(req, &logger.clone(), &config.clone(), &*service).unwrap()

//                return Ok(Response::new(::hyper::Body::from("Hello World")));
                //        if req.version() == Version::HTTP_11 {
                //
                //        } else {
                //            // Note: it's usually better to return a Response
                //            // with an appropriate StatusCode instead of an Err.
                //            Err("not HTTP/1.1, abort connection")
                //        }
            },
        )
    };

    let server = Server::bind(&addr).serve(new_service);

    // Finally, spawn `server` onto an Executor...
    ::hyper::rt::run(server.map_err(|e| {
        eprintln!("server error: {}", e);
    }));

    Ok(())
}
