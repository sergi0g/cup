use check::get_updates;
use clap::{Parser, Subcommand};
use config::Config;
use formatting::spinner::Spinner;
#[cfg(feature = "cli")]
use formatting::{print_raw_updates, print_updates};
#[cfg(feature = "server")]
use server::serve;
use std::path::PathBuf;
use std::time::SystemTime;

pub mod check;
pub mod config;
pub mod docker;
#[cfg(feature = "cli")]
pub mod formatting;
pub mod http;
pub mod registry;
#[cfg(feature = "server")]
pub mod server;
pub mod structs;
pub mod utils;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long, default_value = None)]
    socket: Option<String>,
    #[arg(short, long, default_value_t = String::new(), help = "Config file path")]
    config_path: String,
    #[command(subcommand)]
    command: Option<Commands>,
    #[arg(short, long)]
    debug: bool,
}

#[derive(Subcommand)]
enum Commands {
    #[cfg(feature = "cli")]
    Check {
        #[arg(name = "images", default_value = None)]
        references: Option<Vec<String>>,
        #[arg(short, long, default_value_t = false, help = "Enable icons")]
        icons: bool,
        #[arg(
            short,
            long,
            default_value_t = false,
            help = "Output JSON instead of formatted text"
        )]
        raw: bool,
    },
    #[cfg(feature = "server")]
    Serve {
        #[arg(
            short,
            long,
            default_value_t = 8000,
            help = "Use a different port for the server"
        )]
        port: u16,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let cfg_path = match cli.config_path.as_str() {
        "" => None,
        path => Some(PathBuf::from(path)),
    };
    let mut config = Config::new().load(cfg_path);
    if let Some(socket) = cli.socket {
        config.socket = Some(socket)
    }
    config.debug = cli.debug;
    match &cli.command {
        #[cfg(feature = "cli")]
        Some(Commands::Check {
            references,
            icons,
            raw,
        }) => {
            let start = SystemTime::now();
            match *raw || config.debug {
                true => {
                    let updates = get_updates(references, &config).await;
                    print_raw_updates(&updates);
                }
                false => {
                    let spinner = Spinner::new();
                    let updates = get_updates(references, &config).await;
                    spinner.succeed();
                    print_updates(&updates, icons);
                    info!("✨ Checked {} images in {}ms", updates.len(), start.elapsed().unwrap().as_millis());
                }
            };
        }
        #[cfg(feature = "server")]
        Some(Commands::Serve { port }) => {
            let _ = serve(port, &config).await;
        }
        None => error!("Whoops! It looks like you haven't specified a command to run! Try `cup help` to see available options."),
    }
}
