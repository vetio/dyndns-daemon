use std::net::{Ipv4Addr, SocketAddr};
use slog::Logger;

use errors::*;
use openpgp::SignedMessageBuilder;
use config::Config;

pub trait DnsService {
    fn update(&self, addr: &Ipv4Addr) -> Result<()>;
}

pub struct HetznerClient<S> {
    logger: Logger,
    signed_message_builder: S,
    to_addr: String,
    from_addr: String,
    smtp_addr: SocketAddr,
    username: String,
    password: String,
    hetzner_user: String,
    domain: String,
}

impl<S: SignedMessageBuilder> HetznerClient<S> {
    pub fn new(parent_logger: &Logger, config: &Config, signed_message_builder: S) -> Self {
        let logger = parent_logger.new(
            o!("dns-service" => "hetzner")
        );

        use std::net;

        let mut address = net::lookup_host("smtp.strato.de")
            .unwrap()
            .next()
            .unwrap();
        address.set_port(587);

        HetznerClient {
            logger: logger,
            to_addr: config.to_addr.clone(),
            from_addr: config.from_addr.clone(),
            smtp_addr: address,
            username: config.smtp_username.clone(),
            password: config.smtp_password.clone(),
            hetzner_user: config.hetzner_user.clone(),
            domain: config.domain.clone(),
            signed_message_builder: signed_message_builder,
        }
    }

    fn send_mail(&self, text: &str) -> Result<()> {
        use lettre::email::EmailBuilder;
        use lettre::transport::smtp::SmtpTransportBuilder;
        use lettre::transport::EmailTransport;

        let email = EmailBuilder::new()
            .to(self.to_addr.as_ref())
            .from(self.from_addr.as_ref())
            .subject("Dns Update")
            .text(text)
            .build()
            .chain_err(|| "Error building email")?;

        let mut transport = SmtpTransportBuilder::new(
            &self.smtp_addr
        )
            .chain_err(|| "Error creating transport builder")?
            .credentials(
                self.username.as_ref(),
                self.password.as_ref()
            )
            .connection_reuse(true)
            .build();

        transport.send(email)
            .chain_err(|| "Error sending mail")?;
        Ok(())
    }

    fn build_mail_text(&self, addr: &Ipv4Addr) -> Result<String> {
        let mut text = String::new();
        text.push_str(&format!("user: {}\n", self.hetzner_user));
        text.push_str("job: ns\n");
        text.push_str("task: update\n");
        text.push_str(&format!("domain: {}\n", self.domain));
        text.push_str("primary: yours\n");
        text.push_str("zonefile: /begin\n");

        use chrono::*;

        let now: DateTime<UTC> = UTC::now();

        text.push_str("@ IN SOA ns1.first-ns.de. postmaster.robot.first-ns.de. (\n");
        text.push_str(&format!("        {}; Serial\n", now.timestamp()));
        text.push_str("        86400; Refresh\n");
        text.push_str("        7200; Retry\n");
        text.push_str("        604800; Expire\n");
        text.push_str("        7200); Minimum\n");
        text.push_str("@ IN NS ns1.first-ns.de.\n");
        text.push_str("@ IN NS ns.second-ns.de.\n");
        text.push_str(&format!(" @ IN A {}\n", addr));
        text.push_str(" / end\n");

        self.signed_message_builder.sign(&text)
            .chain_err(|| "Error signing email")
    }
}

impl<S: SignedMessageBuilder> DnsService for HetznerClient<S> {
    fn update(&self, addr: &Ipv4Addr) -> Result<()> {
        info! ( self.logger, "called with: {}", addr);

        let mail_text = self.build_mail_text(addr)
            .chain_err(|| "Error building email text")?;

        self.send_mail(&mail_text).chain_err(|| "Foo")?;

        Ok(())
    }
}

