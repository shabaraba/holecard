use anyhow::Result;

use super::swift_runner::run_swift;

const SERVICE_NAME: &str = "hc";
const MASTER_PASSWORD_PREFIX: &str = "master_password";

fn account_name(vault_name: &str) -> String {
    format!("{}-{}", MASTER_PASSWORD_PREFIX, vault_name)
}

pub fn save_master_password(vault_name: &str, master_password: &str) -> Result<()> {
    let account = account_name(vault_name);
    let escaped_password = master_password.replace('\\', "\\\\").replace('"', "\\\"");

    let script = format!(
        r#"
import Foundation
import Security

let service = "{SERVICE_NAME}" as CFString
let account = "{account}" as CFString
let password = "{escaped_password}".data(using: .utf8)!

var query: [String: Any] = [
    kSecClass as String: kSecClassGenericPassword,
    kSecAttrService as String: service,
    kSecAttrAccount as String: account,
    kSecAttrAccessible as String: kSecAttrAccessibleWhenUnlockedThisDeviceOnly,
    kSecValueData as String: password
]

SecItemDelete(query as CFDictionary)

let status = SecItemAdd(query as CFDictionary, nil)
if status == errSecSuccess {{
    exit(0)
}} else {{
    print("error: \(status)")
    exit(1)
}}
"#
    );

    let output = run_swift(&script)?;
    if !output.success {
        return Err(anyhow::anyhow!(
            "Failed to save password to keychain.\nstdout: {}\nstderr: {}",
            output.stdout,
            output.stderr
        ));
    }
    Ok(())
}

pub fn load_master_password(vault_name: &str) -> Result<Option<String>> {
    let account = account_name(vault_name);

    let script = format!(
        r#"
import Foundation
import Security

let service = "{SERVICE_NAME}" as CFString
let account = "{account}" as CFString

let query: [String: Any] = [
    kSecClass as String: kSecClassGenericPassword,
    kSecAttrService as String: service,
    kSecAttrAccount as String: account,
    kSecReturnData as String: true
]

var item: CFTypeRef?
let status = SecItemCopyMatching(query as CFDictionary, &item)

if status == errSecSuccess, let data = item as? Data, let password = String(data: data, encoding: .utf8) {{
    print(password)
    exit(0)
}} else {{
    exit(1)
}}
"#
    );

    let output = run_swift(&script)?;
    if output.success {
        Ok(Some(output.stdout))
    } else {
        Ok(None)
    }
}

#[allow(dead_code)]
pub fn delete_master_password(vault_name: &str) -> Result<()> {
    let account = account_name(vault_name);

    let script = format!(
        r#"
import Foundation
import Security

let service = "{SERVICE_NAME}" as CFString
let account = "{account}" as CFString

let query: [String: Any] = [
    kSecClass as String: kSecClassGenericPassword,
    kSecAttrService as String: service,
    kSecAttrAccount as String: account,
]

SecItemDelete(query as CFDictionary)
exit(0)
"#
    );

    run_swift(&script)?;
    Ok(())
}
