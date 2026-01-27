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

        #[arg(
            short,
            long,
            value_name = "FIELD",
            help = "Copy field to clipboard (optional field name, defaults to 'password' or first field)"
        )]
        clip: Option<Option<String>>,

        #[arg(long, help = "Show TOTP code")]
        totp: bool,

        #[arg(long, help = "Show field values (requires password re-entry)")]
        show: bool,
    },

    #[command(about = "List all entries")]
    List,

    #[command(about = "Edit an entry")]
    Edit {
        #[arg(help = "Entry name")]
        name: String,

        #[arg(short, long, help = "Interactive mode")]
        interactive: bool,

        #[arg(short, long, value_parser = parse_field, help = "Add or update field (key=value)")]
        field: Vec<(String, String)>,

        #[arg(long, help = "Remove field by key")]
        rm_field: Vec<String>,
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

    #[command(about = "Inject environment variables from template")]
    Inject {
        #[arg(help = "Entry name")]
        entry: String,

        #[arg(help = "Template string with {{entry.field}} or {{entry}}")]
        template: String,
    },

    #[command(about = "Run command with entry environment variables")]
    Run {
        #[arg(help = "Entry name to use for environment variables")]
        entry: String,

        #[arg(last = true, help = "Command and arguments to execute")]
        command: Vec<String>,
    },

    #[command(about = "Lock the vault (clear session)")]
    Lock,

    #[command(about = "Show session status")]
    Status,

    #[command(about = "Export vault to JSON file")]
    Export {
        #[arg(help = "Output file path")]
        file: String,
    },

    #[command(about = "Import entries from JSON file")]
    Import {
        #[arg(help = "Input file path")]
        file: String,

        #[arg(long, help = "Overwrite existing entries")]
        overwrite: bool,
    },

    #[command(about = "Manage TOTP (Time-based One-Time Password)")]
    Totp {
        #[command(subcommand)]
        subcommand: TotpCommands,
    },

    #[command(about = "Manage secret providers (GitHub, Cloudflare, etc.)")]
    Provider {
        #[command(subcommand)]
        subcommand: ProviderCommands,
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

#[derive(Subcommand)]
pub enum ProviderCommands {
    #[command(about = "Add a new provider configuration")]
    Add {
        #[arg(help = "Provider type (github, cloudflare)")]
        provider_type: String,

        #[arg(help = "Provider ID (e.g., my-repo, my-worker)")]
        provider_id: String,

        #[arg(short, long, value_parser = parse_field, help = "Provider credential (key=value)")]
        cred: Vec<(String, String)>,
    },

    #[command(about = "Push secret(s) to provider")]
    Push {
        #[arg(help = "Provider type")]
        provider_type: String,

        #[arg(help = "Provider ID")]
        provider_id: String,

        #[arg(help = "Entry name or entry.field (e.g., myapp.db_url)")]
        entry_field: String,

        #[arg(long, help = "Override secret name in provider")]
        as_name: Option<String>,

        #[arg(long, help = "Push all fields from entry as separate secrets")]
        expand: bool,
    },

    #[command(about = "List all configured providers")]
    List,

    #[command(about = "List secrets in a provider")]
    Secrets {
        #[arg(help = "Provider type")]
        provider_type: String,

        #[arg(help = "Provider ID")]
        provider_id: String,
    },

    #[command(about = "Delete a secret from provider")]
    DeleteSecret {
        #[arg(help = "Provider type")]
        provider_type: String,

        #[arg(help = "Provider ID")]
        provider_id: String,

        #[arg(help = "Secret name to delete")]
        secret_name: String,
    },

    #[command(about = "Remove a provider configuration")]
    Rm {
        #[arg(help = "Provider type")]
        provider_type: String,

        #[arg(help = "Provider ID")]
        provider_id: String,
    },
}

#[derive(Subcommand)]
pub enum TotpCommands {
    #[command(about = "Add TOTP secret to an entry")]
    Add {
        #[arg(help = "Entry name")]
        entry: String,

        #[arg(help = "TOTP secret (base32 encoded)")]
        secret: String,
    },

    #[command(about = "Get TOTP code (displays and copies to clipboard)")]
    Get {
        #[arg(help = "Entry name")]
        entry: String,
    },

    #[command(about = "Remove TOTP secret from an entry")]
    Rm {
        #[arg(help = "Entry name")]
        entry: String,
    },
}

fn parse_field(s: &str) -> Result<(String, String), String> {
    let parts: Vec<&str> = s.splitn(2, '=').collect();
    if parts.len() != 2 {
        return Err(format!("Invalid field format: '{}'. Expected key=value", s));
    }
    Ok((parts[0].to_string(), parts[1].to_string()))
}
