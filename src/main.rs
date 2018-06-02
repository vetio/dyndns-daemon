#![deny(warnings)]
#![allow(renamed_and_removed_lints)]

#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]
#![recursion_limit = "1024"]

extern crate clap;

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
extern crate slog_async;
extern crate slog_term;

#[cfg(feature = "use_dotenv")]
extern crate dotenv;

extern crate itertools;

extern crate consistenttime;

#[cfg(test)]
#[macro_use]
extern crate quickcheck;

mod config;
mod dns;
mod envvars;
mod errors;
mod http;
mod openpgp;
mod template;

use errors::*;

fn run(root_logger: &slog::Logger) -> Result<()> {
    use config::Config;
    use dns::HetznerClient;
    use http::run_server;
    use openpgp::Sha1SignedMessageBuilder;

    envvars::use_dotenv()?;

    let config = Config::new()?;
    debug!(root_logger, "config: {:#?}", config);

    let signed_message_builder = Sha1SignedMessageBuilder::new(&config);

    let dns_service = HetznerClient::new(root_logger, &config, signed_message_builder);

    run_server(root_logger, dns_service, &config).chain_err(|| "Error running server")
}

fn main() {
    let arguments = parse_args().expect("Error processing arguments");

    use slog::Drain;

    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();

    let root_logger = slog::Logger::root(drain, o!("version" => "0.1"));
    info!(root_logger, "Application started");
    debug!(root_logger, "arguments: {:#?}", arguments);

    if let Err(ref e) = run(&root_logger) {
        log_error(&root_logger, e);
        std::process::exit(1);
    }
}

#[derive(Debug)]
struct Args {
    config: Option<String>,
}

fn parse_args() -> Result<Args> {
    use clap::{App, Arg};

    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("Sets the config file. Supports json and yaml.")
                .takes_value(true),
        )
        .get_matches();

    let config = matches.value_of("config").map(ToOwned::to_owned);

    Ok(Args { config })
}
