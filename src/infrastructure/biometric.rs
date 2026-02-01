use anyhow::Result;
use std::process::Command;

use crate::config::Config;

pub trait BiometricAuth {
    fn is_available(&self) -> bool;
    fn authenticate(&self, reason: &str) -> Result<bool>;
}

#[cfg(target_os = "macos")]
pub struct MacOSBiometric;

#[cfg(target_os = "macos")]
impl BiometricAuth for MacOSBiometric {
    fn is_available(&self) -> bool {
        Command::new("security")
            .arg("authorizationdb")
            .arg("read")
            .arg("system.login.done")
            .output()
            .is_ok()
    }

    fn authenticate(&self, reason: &str) -> Result<bool> {
        let output = Command::new("osascript")
            .arg("-e")
            .arg(format!(
                r#"display dialog "{}" with title "holecard Authentication" buttons {{"Cancel", "Authenticate"}} default button "Authenticate""#,
                reason
            ))
            .output()?;

        if !output.status.success() {
            return Ok(false);
        }

        let auth_output = Command::new("security")
            .arg("execute-with-privileges")
            .arg("/usr/bin/true")
            .output();

        match auth_output {
            Ok(result) => Ok(result.status.success()),
            Err(_) => {
                let fallback = Command::new("security")
                    .arg("authorize")
                    .output()?;
                Ok(fallback.status.success())
            }
        }
    }
}

#[cfg(not(target_os = "macos"))]
pub struct StubBiometric;

#[cfg(not(target_os = "macos"))]
impl BiometricAuth for StubBiometric {
    fn is_available(&self) -> bool {
        false
    }

    fn authenticate(&self, _reason: &str) -> Result<bool> {
        Ok(false)
    }
}

pub fn get_biometric_auth() -> Box<dyn BiometricAuth> {
    #[cfg(target_os = "macos")]
    {
        Box::new(MacOSBiometric)
    }

    #[cfg(not(target_os = "macos"))]
    {
        Box::new(StubBiometric)
    }
}

pub fn require_biometric_auth(config: &Config, reason: &str) -> Result<()> {
    if !config.enable_biometric {
        return Ok(());
    }

    let biometric = get_biometric_auth();
    if !biometric.is_available() {
        return Ok(());
    }

    println!("🔐 Touch ID authentication required...");
    match biometric.authenticate(reason) {
        Ok(true) => {
            println!("✅ Authenticated");
            Ok(())
        }
        Ok(false) => Err(anyhow::anyhow!("Touch ID authentication failed")),
        Err(e) => Err(anyhow::anyhow!("Touch ID error: {}", e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_biometric_availability() {
        let auth = get_biometric_auth();
        let _ = auth.is_available();
    }
}
