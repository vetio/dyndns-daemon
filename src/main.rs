#![deny(warnings)]
#![allow(zero_ptr)] // Necessary for lazy_static

#![cfg_attr(test, feature(plugin))]
#![cfg_attr(test, plugin(quickcheck_macros))]
#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]

#![recursion_limit = "1024"]

#![feature(slice_patterns)]
#![feature(lookup_host)]

#[macro_use]
extern crate error_chain;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate serde_derive;

extern crate lettre;

extern crate chrono;

extern crate hyper;

extern crate envy;


#[macro_use]
extern crate slog;
extern crate slog_term;

extern crate dotenv;

extern crate itertools;

extern crate consistenttime;

#[cfg(test)]
extern crate quickcheck;

mod errors;
mod openpgp;
mod http;
mod dns;
mod config;
mod template;

use errors::*;

lazy_static! {
    static ref IS_DEBUG: bool = {
        use std::env;

        match env::vars().find(|&(ref key, _)| key == "DEVEL") {
            Some((_, val)) => val == "1",
            None => false,
        }
    };
}

fn run(root_logger: &slog::Logger) -> Result<()> {
    use http::run_server;
    use dns::HetznerClient;
    use config::Config;
    use openpgp::Sha1SignedMessageBuilder;

    if *IS_DEBUG {
        if let Err(ref e) = dotenv::dotenv() {
            bail!("failed to load .env file: {:?}", e);
        }
    }

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
    use slog::DrainExt;

    let drain = slog_term::streamer().build().fuse();
    let root_logger = slog::Logger::root(
        drain, o!("version" => "0.1")
    );
    info!(root_logger, "Application started");

    if let Err(ref e) = run(&root_logger) {
        log_error(&root_logger, e);
        std::process::exit(1);
    }
}
