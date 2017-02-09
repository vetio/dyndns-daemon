use std::net::SocketAddr;
use errors::*;

#[derive(Deserialize, Debug)]
struct RawConfig {
    from_addr: String,
    to_addr: String,
    smtp_host: String,
    smtp_port: u16,
    smtp_username: String,
    smtp_password: String,
    pgp_key: String,
    domain: String,
    hetzner_user: String,
}

impl RawConfig {
    fn smtp_addr(&self) -> Result<SocketAddr> {
        use std::net;

        let mut address = net::lookup_host(&self.smtp_host)
            .chain_err(
                || format!(
                    "Error resolving host: {}",
                    self.smtp_host
                )
            )?
            .next()
            .ok_or_else(
                || format!(
                    "No such host: {}",
                    self.smtp_host
                )
            )?;

        address.set_port(self.smtp_port);

        Ok(address)
    }
}

#[derive(Debug)]
pub struct Config {
    pub from_addr: String,
    pub to_addr: String,
    pub smtp_addr: SocketAddr,
    pub smtp_username: String,
    pub smtp_password: String,
    pub pgp_key: String,
    pub domain: String,
    pub hetzner_user: String,
}

impl Config {
    pub fn new() -> Result<Config> {
        use envy::from_env;

        let raw_config: RawConfig = from_env()
            .chain_err(|| "Failed to load environment config")?;
        let address = raw_config.smtp_addr()?;

        Ok(Config {
            from_addr: raw_config.from_addr,
            to_addr: raw_config.to_addr,
            smtp_addr: address,
            smtp_username: raw_config.smtp_username,
            smtp_password: raw_config.smtp_password,
            pgp_key: raw_config.pgp_key,
            domain: raw_config.domain,
            hetzner_user: raw_config.hetzner_user,
        })
    }
}
