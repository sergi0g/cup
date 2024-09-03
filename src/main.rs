#[cfg(feature = "cli")]
use check::{get_all_updates, get_update};
use clap::{Parser, Subcommand};
#[cfg(feature = "cli")]
use formatting::{print_raw_update, print_raw_updates, print_update, print_updates, Spinner};
#[cfg(feature = "server")]
use server::serve;
use std::path::PathBuf;
use utils::load_config;

pub mod check;
pub mod docker;
#[cfg(feature = "cli")]
pub mod formatting;
pub mod image;
pub mod registry;
#[cfg(feature = "server")]
pub mod server;
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
}

#[derive(Subcommand)]
enum Commands {
    #[cfg(feature = "cli")]
    Check {
        #[arg(default_value = None)]
        image: Option<String>,
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
    let config = load_config(cfg_path);
    match &cli.command {
        #[cfg(feature = "cli")]
        Some(Commands::Check { image, icons, raw }) => match image {
            Some(name) => {
                let has_update = get_update(name, cli.socket, &config).await;
                match raw {
                    true => print_raw_update(name, &has_update),
                    false => print_update(name, &has_update),
                };
            }
            None => {
                match raw {
                    true => print_raw_updates(&get_all_updates(cli.socket, &config).await),
                    false => {
                        let spinner = Spinner::new();
                        let updates = get_all_updates(cli.socket, &config).await;
                        spinner.succeed();
                        print_updates(&updates, icons);
                    }
                };
            }
        },
        #[cfg(feature = "server")]
        Some(Commands::Serve { port }) => {
            let _ = serve(port, cli.socket, config).await;
        }
        None => (),
    }
}
