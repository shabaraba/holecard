use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "hc")]
#[command(about = "Secure password manager CLI", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Initialize a new vault")]
    Init,

    #[command(about = "Add a new entry")]
    Add {
        #[arg(help = "Entry name")]
        name: Option<String>,

        #[arg(short, long, value_parser = parse_field, help = "Custom field (key=value)")]
        field: Vec<(String, String)>,
    },

    #[command(about = "Get an entry")]
    Get {
        #[arg(help = "Entry name")]
        name: String,

        #[arg(short, long, help = "Copy to clipboard")]
        clip: bool,
    },

    #[command(about = "List all entries")]
    List,

    #[command(about = "Edit an entry")]
    Edit {
        #[arg(help = "Entry name")]
        name: String,
    },

    #[command(about = "Remove an entry")]
    Rm {
        #[arg(help = "Entry name")]
        name: String,
    },

    #[command(about = "Manage configuration")]
    Config {
        #[command(subcommand)]
        subcommand: Option<ConfigCommands>,
    },
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    #[command(about = "Set vault file path")]
    VaultPath {
        #[arg(help = "New vault file path")]
        path: String,
    },

    #[command(about = "Set session timeout (minutes)")]
    SessionTimeout {
        #[arg(help = "Timeout in minutes")]
        minutes: u64,
    },
}

fn parse_field(s: &str) -> Result<(String, String), String> {
    let parts: Vec<&str> = s.splitn(2, '=').collect();
    if parts.len() != 2 {
        return Err(format!("Invalid field format: '{}'. Expected key=value", s));
    }
    Ok((parts[0].to_string(), parts[1].to_string()))
}
