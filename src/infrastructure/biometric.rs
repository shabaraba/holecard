use anyhow::Result;

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
        use security_framework::os::macos::keychain::SecKeychain;
        SecKeychain::default().is_ok()
    }

    fn authenticate(&self, reason: &str) -> Result<bool> {
        use std::process::Command;
        use std::io::Write;

        let swift_code = format!(
            r#"
import Foundation
import LocalAuthentication

let context = LAContext()
var error: NSError?

guard context.canEvaluatePolicy(.deviceOwnerAuthentication, error: &error) else {{
    print("false")
    exit(1)
}}

let semaphore = DispatchSemaphore(value: 0)
var authResult = false

context.evaluatePolicy(.deviceOwnerAuthentication, localizedReason: "{}") {{ success, authError in
    authResult = success
    semaphore.signal()
}}

semaphore.wait()
print(authResult ? "true" : "false")
exit(authResult ? 0 : 1)
"#,
            reason.replace("\"", "\\\"")
        );

        let mut child = Command::new("swift")
            .arg("-")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(swift_code.as_bytes())?;
        }

        let output = child.wait_with_output()?;
        let result = String::from_utf8_lossy(&output.stdout).trim().to_string();

        Ok(output.status.success() && result == "true")
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

    println!("🔐 Authentication required...");
    match biometric.authenticate(reason) {
        Ok(true) => {
            println!("✅ Authenticated");
            Ok(())
        }
        Ok(false) => Err(anyhow::anyhow!("Authentication failed")),
        Err(e) => Err(anyhow::anyhow!("Authentication error: {}", e)),
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
