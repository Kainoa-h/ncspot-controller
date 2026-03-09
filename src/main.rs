mod cli;
mod config;
mod control;
mod monitor;
mod ncspot_event;
mod socket;

use clap::Parser;
use cli::Cli;
use control::execute_command;
use monitor::run_monitor;
use socket::get_ncspot_socket_path;

fn main() {
    let cli = Cli::parse();
    let config = config::Config::load();

    let socket_path = match get_ncspot_socket_path(&config.ncspot_binary) {
        Ok(path) => path,
        Err(e) => {
            println!("Failed to get ncspot socket path: {}", e);
            let binary = config.ncspot_binary.as_deref().unwrap_or("ncspot");
            println!("Make sure {} is installed and accessible", binary);
            std::process::exit(1);
        }
    };

    match &cli.command {
        Some(cmd) => {
            if let Err(e) = execute_command(&socket_path, cmd) {
                println!("Failed to send command: {}", e);
                std::process::exit(1);
            }
        }
        None => run_monitor(&socket_path, &config),
    }
}
