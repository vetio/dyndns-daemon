use errors::*;
use template::Template;
use std::path::Path;

static IP_RESOLV_METHOD_DYNDNS2: &str = "DynDns2";
static IP_RESOLV_METHOD_HEADER: &str = "Header";

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
    ip_resolv_method: String,
    ip_header: Option<String>,
    template: String,
}

impl RawConfig {
    fn from_env() -> Result<Self> {
        use envy::from_env;
        from_env().chain_err(|| "Failed to load environment config")
    }

    fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        use toml::de::from_str;
        use std::io::Read;
        use std::fs::File;

        let mut config = String::new();
        let mut file: File = File::open(path).chain_err(|| "Error opening config file")?;
        file.read_to_string(&mut config).chain_err(|| "Error reading from config file")?;

        from_str(&config).chain_err(|| "Error parsing config file")
    }

    fn get_template(&self) -> Result<Template> {
        use std::fs;
        use std::io::Read;

        let mut file = fs::File::open(&self.template).chain_err(|| "Error opening template")?;
        let mut buffer = String::new();
        file.read_to_string(&mut buffer)
            .chain_err(|| "Error reading template to String")?;

        Ok(Template::from(&buffer as &str))
    }

    fn get_ip_resolv(&self) -> Result<IpResolvMethod> {
        match self.ip_resolv_method {
            ref m if m == IP_RESOLV_METHOD_DYNDNS2 => Ok(IpResolvMethod::DynDns2),
            ref m if m == IP_RESOLV_METHOD_HEADER => match &self.ip_header {
                Some(header_name) => Ok(IpResolvMethod::Header(header_name.clone())),
                None => Err("IP_HEADER not set.".into()),
            },
            _ => Err(format!(
                "Unknown IP_RESOLV variant. Supported: {}, {}",
                IP_RESOLV_METHOD_DYNDNS2, IP_RESOLV_METHOD_HEADER
            ).into()),
        }
    }
}

#[derive(Debug, Clone)]
pub enum IpResolvMethod {
    Header(String),
    DynDns2,
}

#[derive(Debug)]
pub struct Config {
    pub from_addr: String,
    pub to_addr: String,
    pub smtp_host: String,
    pub smtp_username: String,
    pub smtp_password: String,
    pub pgp_key: String,
    pub domain: String,
    pub hetzner_user: String,
    pub server_addr: String,
    pub http_auth_user: String,
    pub http_auth_password: String,
    pub ip_resolv: IpResolvMethod,
    pub template: Template,
}

impl Config {
    pub fn from_source(source: &::ConfigSource) -> Result<Config> {
        let raw_config = match source {
            ::ConfigSource::Env => RawConfig::from_env(),
            ::ConfigSource::File(path) => RawConfig::from_file(path)
                .chain_err(|| format!("Error reading config from {}", path.to_string_lossy())),
        }?;

        let template = raw_config
            .get_template()
            .chain_err(|| "Error evaluating template")?;
        let ip_resolv = raw_config
            .get_ip_resolv()
            .chain_err(|| "Error parsing ip resolution")?;

        Ok(Config {
            from_addr: raw_config.from_addr,
            to_addr: raw_config.to_addr,
            smtp_host: raw_config.smtp_host,
            smtp_username: raw_config.smtp_username,
            smtp_password: raw_config.smtp_password,
            pgp_key: raw_config.pgp_key,
            domain: raw_config.domain,
            hetzner_user: raw_config.hetzner_user,
            server_addr: raw_config.server_addr,
            http_auth_user: raw_config.http_auth_user,
            http_auth_password: raw_config.http_auth_password,
            ip_resolv,
            template,
        })
    }
}
