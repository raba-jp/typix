#[macro_use]
extern crate log;
#[macro_use]
extern crate anyhow;

use anyhow::Context as _;
pub use chrono::prelude::*;
use serde::Deserialize;
use std::path::PathBuf;
use structopt::StructOpt;

mod input;
mod pixela;

#[derive(Debug, StructOpt)]
#[structopt(name = "typix")]
pub struct App {
    #[structopt(subcommand)]
    pub command: Command,
    #[structopt(
        short = "c",
        long,
        parse(from_os_str),
        default_value = "~/.config/tyco/config.toml"
    )]
    pub config: PathBuf,
}

#[derive(Debug, StructOpt)]
pub enum Command {
    #[structopt(name = "list")]
    List,
    #[structopt(name = "listen")]
    Listen,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    version: i64,
    username: String,
    token: String,
    graph_id: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut builder = env_logger::Builder::new();
    if cfg!(debug_assertions) {
        builder.filter_level(log::LevelFilter::Debug).init();
    }

    let app = App::from_args();

    let cfg = load_configuration(&app.config).context("Failed to load configuration")?;

    match app.command {
        Command::List => list()?,
        Command::Listen => listen(&cfg).await?,
    }
    Ok(())
}

fn load_configuration(config_path: &PathBuf) -> anyhow::Result<Config> {
    let file_contents: String;
    if config_path.is_file() {
        debug!("configuration file path: {}", config_path.display());
        file_contents = std::fs::read_to_string(config_path)?;
    } else {
        let base = directories::BaseDirs::new().context("Failed to get home directory")?;
        let home = base.config_dir().join("tyco/config.toml");
        debug!("configuration file path: {}", &home.display());
        file_contents = std::fs::read_to_string(home).context("Failed to read file")?;
    }
    let cfg: Config = toml::from_str(&file_contents)?;
    Ok(cfg)
}

fn list() -> anyhow::Result<()> {
    let devices = input::devices().with_context(|| "failed to get devices")?;
    for device in devices {
        println!("{} {}", device.0.display(), device.1.name().unwrap_or(""));
    }
    Ok(())
}

async fn listen(cfg: &Config) -> anyhow::Result<()> {
    let api = pixela::API::new(cfg.username.to_owned(), cfg.token.to_owned());
    let graph_id = cfg.graph_id.to_owned();
    let (tx, rx) = std::sync::mpsc::channel::<i64>();

    tokio::task::spawn(async move {
        debug!("test");
        let mut count: i64 = 0;
        count += api.get_pixel(&graph_id).await;
        loop {
            let _ = rx.recv().unwrap();
            count += 1;
            if count % 50 == 0 {
                match api.post_pixel(&graph_id, count).await {
                    Ok(_) => {}
                    Err(_) => {}
                }
            }
        }
    });

    let path = input::select_device()?;
    let file = std::fs::File::open(path).unwrap();
    input::listen(file, tx)
        .with_context(|| "failed to listen device")
        .unwrap();

    Ok(())
}
