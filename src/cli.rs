use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "ncspot-controller")]
#[command(about = "Monitor and control ncspot", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Play the current track
    Play,
    /// Pause playback
    Pause,
    /// Toggle play/pause
    Playpause,
    /// Skip to next track
    Next,
    /// Go to previous track
    Previous,
    /// Stop playback
    Stop,
    /// Send a raw command to ncspot
    Raw { command: String },
}
