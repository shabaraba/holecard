use anyhow::{bail, Context, Result};
use std::io::Write;
use std::process::{Command, Output, Stdio};
use tempfile::NamedTempFile;

pub struct SshAgent;

impl SshAgent {
    pub fn connect() -> Result<Self> {
        if std::env::var("SSH_AUTH_SOCK").is_err() {
            bail!("ssh-agent is not running. Start it with 'eval $(ssh-agent -s)'");
        }

        let output = run_ssh_add(&["-l"])?;

        if !output.status.success() && output.status.code() != Some(1) {
            bail!("ssh-agent is not accessible");
        }

        Ok(Self)
    }

    pub fn add_identity(
        &self,
        private_key: &str,
        passphrase: Option<&str>,
        lifetime: Option<u32>,
    ) -> Result<()> {
        let mut temp_file = NamedTempFile::new().context("Failed to create temporary file")?;

        temp_file
            .write_all(private_key.as_bytes())
            .context("Failed to write private key to temp file")?;
        temp_file.flush()?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(temp_file.path(), std::fs::Permissions::from_mode(0o600))?;
        }

        let mut cmd = Command::new("ssh-add");

        if let Some(sec) = lifetime {
            cmd.arg("-t").arg(sec.to_string());
        }

        cmd.arg(temp_file.path());

        let output = if let Some(pass) = passphrase {
            cmd.stdin(Stdio::piped());
            let mut child = cmd.spawn().context("Failed to spawn ssh-add")?;
            if let Some(mut stdin) = child.stdin.take() {
                writeln!(stdin, "{}", pass)?;
            }
            child.wait_with_output()?
        } else {
            cmd.output().context("Failed to execute ssh-add")?
        };

        check_ssh_add_output(&output, "add")?;
        Ok(())
    }

    pub fn remove_identity(&self, public_key: &str) -> Result<()> {
        let output = run_ssh_add(&["-d", public_key])?;
        check_ssh_add_output(&output, "remove")?;
        Ok(())
    }

    pub fn list_identities(&self) -> Result<Vec<String>> {
        let output = run_ssh_add(&["-l"])?;

        if output.status.code() == Some(1) {
            return Ok(vec![]);
        }

        if !output.status.success() {
            bail!("Failed to list SSH keys");
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.lines().map(|s| s.to_string()).collect())
    }
}

fn run_ssh_add(args: &[&str]) -> Result<Output> {
    Command::new("ssh-add")
        .args(args)
        .output()
        .with_context(|| format!("Failed to execute ssh-add {}", args.join(" ")))
}

fn check_ssh_add_output(output: &Output, action: &str) -> Result<()> {
    if !output.status.success() {
        bail!(
            "Failed to {} SSH key: {}",
            action,
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(())
}
