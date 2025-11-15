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

    let socket_path = match get_ncspot_socket_path() {
        Ok(path) => path,
        Err(e) => {
            println!("Failed to get ncspot socket path: {}", e);
            println!("Make sure ncspot is installed and accessible in your PATH");
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
        None => run_monitor(&socket_path),
    }
}
