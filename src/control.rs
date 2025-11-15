use crate::cli::Commands;
use crate::socket::send_command;
use std::path::PathBuf;

pub fn execute_command(
    socket_path: &PathBuf,
    cmd: &Commands,
) -> Result<(), Box<dyn std::error::Error>> {
    let command_str = match cmd {
        Commands::Play => "play",
        Commands::Pause => "pause",
        Commands::Playpause => "playpause",
        Commands::Next => "next",
        Commands::Previous => "previous",
        Commands::Stop => "stop",
        Commands::Raw { command } => command.as_str(),
    };
    send_command(socket_path, command_str)
}
