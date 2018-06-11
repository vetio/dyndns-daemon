#![deny(warnings)]
#![allow(renamed_and_removed_lints)]
#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]
#![recursion_limit = "1024"]

extern crate base64;
extern crate clap;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate serde_derive;
extern crate lettre;
extern crate chrono;
extern crate http;
extern crate hyper;
extern crate url;
extern crate envy;
#[macro_use]
extern crate slog;
extern crate slog_async;
extern crate slog_term;
#[cfg(feature = "use_dotenv")]
extern crate dotenv;
extern crate itertools;
extern crate consistenttime;
extern crate toml;

#[cfg(test)]
#[macro_use]
extern crate quickcheck;

mod config;
mod dns;
mod envvars;
mod errors;
mod openpgp;
mod server;
mod template;

use errors::*;

fn run(args: Args, root_logger: &slog::Logger) -> Result<()> {
    use config::Config;
    use dns::HetznerClient;
    use openpgp::Sha1SignedMessageBuilder;
    use server::run_server;

    use std::sync::Arc;

    envvars::use_dotenv()?;

    let config = Config::from_source(&args.config)?;
    debug!(root_logger, "config: {:#?}", config);

    let signed_message_builder = Sha1SignedMessageBuilder::new(&config);

    let dns_service = HetznerClient::new(root_logger, &config, signed_message_builder);

    run_server(root_logger, dns_service, Arc::new(config)).chain_err(|| "Error running server")
}

fn main() {
    let args = parse_args();

    use slog::Drain;

    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();

    let root_logger = slog::Logger::root(drain, o!("version" => env!("CARGO_PKG_VERSION")));
    info!(root_logger, "Application started");
    debug!(root_logger, "arguments: {:#?}", args);

    if let Err(ref e) = run(args, &root_logger) {
        eprintln!("{:?}", e);
        log_error(&root_logger, e);
        std::process::exit(1);
    }
}

#[derive(Debug)]
pub enum ConfigSource {
    Env,
    File(::std::path::PathBuf),
}

#[derive(Debug)]
struct Args {
    config: ConfigSource,
}

fn parse_args() -> Args {
    use clap::{App, Arg};
    use std::path::PathBuf;

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
                .takes_value(true)
                .validator_os(|file| {
                    let path = PathBuf::from(file);
                    if path.exists() { Ok(()) } else {
                        let mut err_msg = file.to_owned();
                        err_msg.push(" does not exist");
                        Err(err_msg)
                    }
                }),
        )
        .get_matches();

    let config = matches.value_of("config")
        .map(PathBuf::from)
        .map(|path| ConfigSource::File(path))
        .unwrap_or(ConfigSource::Env);

    Args { config }
}
