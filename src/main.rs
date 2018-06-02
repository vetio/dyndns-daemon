#![deny(warnings)]
#![allow(renamed_and_removed_lints)]

#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]

#![recursion_limit = "1024"]

#[macro_use]
extern crate error_chain;

#[macro_use]
extern crate serde_derive;

extern crate lettre;

extern crate chrono;

extern crate hyper;

extern crate envy;


#[macro_use]
extern crate slog;
extern crate slog_term;
extern crate slog_async;

#[cfg(feature = "use_dotenv")]
extern crate dotenv;

extern crate itertools;

extern crate consistenttime;

#[cfg(test)]
#[macro_use]
extern crate quickcheck;

mod envvars;
mod errors;
mod openpgp;
mod http;
mod dns;
mod config;
mod template;

use errors::*;

fn run(root_logger: &slog::Logger) -> Result<()> {
    use http::run_server;
    use dns::HetznerClient;
    use config::Config;
    use openpgp::Sha1SignedMessageBuilder;

    envvars::use_dotenv()?;

    let config = Config::new()?;
    debug!(root_logger, "config: {:?}", config);

    let signed_message_builder = Sha1SignedMessageBuilder::new(
        &config
    );

    let dns_service = HetznerClient::new(
        root_logger,
        &config,
        signed_message_builder
    );

    run_server(root_logger, dns_service, &config)
        .chain_err(|| "Error running server")
}

fn main() {
    use slog::Drain;

    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();

    let root_logger = slog::Logger::root(
        drain, o!("version" => "0.1")
    );
    info!(root_logger, "Application started");

    if let Err(ref e) = run(&root_logger) {
        log_error(&root_logger, e);
        std::process::exit(1);
    }
}
