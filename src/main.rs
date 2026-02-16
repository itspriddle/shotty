mod backend;
mod cli;
mod clipboard;
mod config;
mod credentials;
mod error;
mod input;

use anyhow::Result;
use clap::Parser;

use cli::{Cli, Command, ConfigAction};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Upload { file, name, copy } => {
            let config = config::Config::load()?;
            let backend = backend::create_backend(&config)?;

            let file_input = input::resolve_input(file.as_deref(), name.as_deref())?;

            let url = backend
                .upload(&file_input.filename, &file_input.data)
                .await?;

            if copy {
                clipboard::copy_to_clipboard(&url)?;
                eprintln!("URL copied to clipboard.");
            }

            println!("{url}");
        }

        Command::Auth => {
            let config = config::Config::load()?;
            let backend = backend::create_backend(&config)?;
            backend.authenticate().await?;
        }

        Command::Config { action } => match action {
            ConfigAction::Show => {
                let config = config::Config::load()?;
                print!("{}", config.show()?);
            }
            ConfigAction::Path => {
                println!("{}", config::Config::path().display());
            }
            ConfigAction::Set { key, value } => {
                let mut config = config::Config::load()?;
                config.set(&key, &value)?;
                eprintln!("Set {key} = {value}");
            }
        },
    }

    Ok(())
}
