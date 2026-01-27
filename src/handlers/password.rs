use anyhow::Result;
use copypasta::{ClipboardContext, ClipboardProvider};
use std::thread;
use std::time::Duration;

use crate::domain::PasswordService;

#[allow(clippy::too_many_arguments)]
pub fn handle_generate(
    length: Option<usize>,
    memorable: bool,
    words: Option<usize>,
    no_uppercase: bool,
    no_lowercase: bool,
    no_digits: bool,
    no_symbols: bool,
    clip: bool,
) -> Result<()> {
    let password = PasswordService::generate_from_cli(
        memorable,
        words,
        length,
        no_uppercase,
        no_lowercase,
        no_digits,
        no_symbols,
    )?;

    if clip {
        copy_to_clipboard_with_clear(&password)?;
        println!("Password copied to clipboard (will clear in 30 seconds)");
    } else {
        println!("{}", password);
    }

    Ok(())
}

pub fn copy_to_clipboard_with_clear(value: &str) -> Result<()> {
    let mut ctx = ClipboardContext::new()
        .map_err(|e| anyhow::anyhow!("Failed to initialize clipboard: {:?}", e))?;
    ctx.set_contents(value.to_string())
        .map_err(|e| anyhow::anyhow!("Failed to copy to clipboard: {:?}", e))?;

    thread::spawn(move || {
        thread::sleep(Duration::from_secs(30));
        if let Ok(mut ctx) = ClipboardContext::new() {
            let _ = ctx.set_contents(String::new());
        }
    });

    Ok(())
}
