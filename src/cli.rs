use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "shotty", about = "Upload screenshots and get direct URLs")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Upload a file and print its direct URL
    Upload {
        /// Path to file to upload (reads from stdin if omitted)
        file: Option<String>,

        /// Filename to use when reading from stdin
        #[arg(long)]
        name: Option<String>,

        /// Copy the URL to clipboard after upload
        #[arg(long)]
        copy: bool,
    },

    /// Authenticate with the configured backend
    Auth,

    /// Manage configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(Subcommand)]
pub enum ConfigAction {
    /// Print current configuration
    Show,

    /// Print the config file path
    Path,

    /// Set a config value
    Set {
        /// Config key (e.g., backend, dropbox.app_key)
        key: String,

        /// Value to set
        value: String,
    },
}
