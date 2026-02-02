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
        let script = r#"
import Foundation
import LocalAuthentication

let context = LAContext()
var error: NSError?
let available = context.canEvaluatePolicy(.deviceOwnerAuthentication, error: &error)
print(available ? "true" : "false")
exit(available ? 0 : 1)
"#;
        super::swift_runner::run_swift(script)
            .map(|o| o.success)
            .unwrap_or(false)
    }

    fn authenticate(&self, reason: &str) -> Result<bool> {
        let escaped_reason = reason.replace('"', "\\\"");
        let script = format!(
            r#"
import Foundation
import LocalAuthentication

let context = LAContext()
var error: NSError?

guard context.canEvaluatePolicy(.deviceOwnerAuthentication, error: &error) else {{
    exit(1)
}}

let semaphore = DispatchSemaphore(value: 0)
var authResult = false

context.evaluatePolicy(.deviceOwnerAuthentication, localizedReason: "{escaped_reason}") {{ success, _ in
    authResult = success
    semaphore.signal()
}}

semaphore.wait()
exit(authResult ? 0 : 1)
"#
        );

        let output = super::swift_runner::run_swift(&script)?;
        Ok(output.success)
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
