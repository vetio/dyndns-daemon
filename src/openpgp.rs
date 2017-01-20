use errors::*;
use config::Config;

pub trait SignedMessageBuilder: Send + Sync + 'static {
    fn sign(&self, text: &str) -> Result<String>;
}

pub struct Sha1SignedMessageBuilder {
    key: String,
}

impl Sha1SignedMessageBuilder {
    pub fn new(config: &Config) -> Self {
        Sha1SignedMessageBuilder {
            key: config.pgp_key.clone(),
        }
    }
}

impl SignedMessageBuilder for Sha1SignedMessageBuilder {
    fn sign(&self, text: &str) -> Result<String> {
        use std::io::Write;
        use std::process::{Command, Stdio};

        let mut cmd = Command::new("gpg");

        // not interactive and no output to terminal
        cmd.arg("--batch");
        cmd.arg("--no-tty");

        // armor
        cmd.arg("-a");

        // stdout
        cmd.arg("-o").arg("-");

        // key
        cmd.arg("--default-key").arg(&self.key);

        // cleartext signature
        cmd.arg("--clearsign");

        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let mut child = cmd.spawn()
            .chain_err(|| "Error spawning gpg")?;

        {
            let mut stdin = child.stdin.as_mut()
                .ok_or_else(|| "Error retrieving stdin for gpg")?;
            stdin.write_all(text.as_bytes())
                .chain_err(|| "Error writing text to stdin")?;
        }

        let output = child.wait_with_output()
            .chain_err(|| "Error waiting for output")?;

        match output.status.code() {
            Some(0) => (),
            Some(i) => bail!("GPG exited with code: {}\nstderr: {}", i, String::from_utf8(output.stderr).unwrap_or_else(|_| "".to_owned())),
            None => bail!("No exit code"),
        };

        String::from_utf8(output.stdout)
            .chain_err(|| "invalid utf - 8 in signed message")
    }
}