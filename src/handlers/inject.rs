use anyhow::{Context, Result};
use std::io::Read;
use std::path::Path;

use crate::domain::SecretResolver;
use crate::infrastructure::KeyringManager;

pub fn handle_inject(
    template: Option<String>,
    input: Option<String>,
    output: Option<String>,
    vault_name: Option<&str>,
    keyring: &KeyringManager,
    config_dir: &Path,
) -> Result<()> {
    let template_str = match (input.as_deref(), template.as_deref()) {
        (Some("-"), None) => {
            let mut buffer = String::new();
            std::io::stdin()
                .read_to_string(&mut buffer)
                .context("Failed to read from stdin")?;
            buffer
        }
        (Some(path), None) => {
            std::fs::read_to_string(path)
                .with_context(|| format!("Failed to read template from {}", path))?
        }
        (None, Some(tpl)) => tpl.to_string(),
        (Some(_), Some(_)) => {
            anyhow::bail!("Cannot specify both --input and template string");
        }
        (None, None) => {
            anyhow::bail!("Must specify either --input or template string");
        }
    };

    let rendered = SecretResolver::resolve_template(&template_str, vault_name, keyring, config_dir)?;

    if let Some(output_path) = output {
        std::fs::write(&output_path, rendered.as_bytes())
            .with_context(|| format!("Failed to write to {}", output_path))?;
        println!("✓ Rendered template written to {}", output_path);
    } else {
        println!("{}", rendered);
    }

    Ok(())
}
