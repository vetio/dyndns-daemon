use std::net::SocketAddr;
use errors::*;
use template::Template;

#[derive(Deserialize, Debug)]
struct RawConfig {
    from_addr: String,
    to_addr: String,
    smtp_host: String,
    smtp_username: String,
    smtp_password: String,
    pgp_key: String,
    domain: String,
    hetzner_user: String,
    server_addr: String,
    http_auth_user: String,
    http_auth_password: String,
    ip_header: String,
    template: String,
}

impl RawConfig {
    fn smtp_addr(&self) -> Result<SocketAddr> {
        use std::net::ToSocketAddrs;

        let address = self.smtp_host.to_socket_addrs()
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

        Ok(address)
    }

    fn get_template(&self) -> Result<Template> {
        use std::fs;
        use std::io::Read;

        let mut file = fs::File::open(&self.template)
            .chain_err(|| "Error opening template")?;
        let mut buffer = String::new();
        file.read_to_string(&mut buffer)
            .chain_err(|| "Error reading template to String")?;

        Ok(Template::from(&buffer as &str))
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
    pub server_addr: String,
    pub http_auth_user: String,
    pub http_auth_password: String,
    pub ip_header: String,
    pub template: Template,
}

impl Config {
    pub fn new() -> Result<Config> {
        use envy::from_env;

        let raw_config: RawConfig = from_env()
            .chain_err(|| "Failed to load environment config")?;
        let address = raw_config.smtp_addr()
            .chain_err(|| "Failed to resolve SMTP addres")?;
        let template = raw_config.get_template()
            .chain_err(|| "Error evaluating template")?;

        Ok(Config {
            from_addr: raw_config.from_addr,
            to_addr: raw_config.to_addr,
            smtp_addr: address,
            smtp_username: raw_config.smtp_username,
            smtp_password: raw_config.smtp_password,
            pgp_key: raw_config.pgp_key,
            domain: raw_config.domain,
            hetzner_user: raw_config.hetzner_user,
            server_addr: raw_config.server_addr,
            http_auth_user: raw_config.http_auth_user,
            http_auth_password: raw_config.http_auth_password,
            ip_header: raw_config.ip_header,
            template: template,
        })
    }
}
