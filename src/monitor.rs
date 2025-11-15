use crate::ncspot_event::{Mode, NcspotEvent};
use crate::socket::wait_for_socket;
use std::io::{BufRead, BufReader};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

fn handle_event(event: NcspotEvent) {
    if let Some(playable) = event.playable {
        let state = match &event.mode {
            Mode::Playing { .. } => "playing",
            Mode::Paused { .. } => "paused",
            Mode::Simple(s) => match s.as_str() {
                "Stopped" => "stopped",
                "FinishedTrack" => "finished",
                _ => "unknown",
            },
        };

        // Output format: state|artist|title|album
        // Status bars can easily parse this with field separators
        println!(
            "{}|{}|{}|{}",
            state,
            playable.artists.join(", "),
            playable.title,
            playable.album
        );
    } else {
        let state = match &event.mode {
            Mode::Playing { .. } => "playing",
            Mode::Paused { .. } => "paused",
            Mode::Simple(s) => match s.as_str() {
                "Stopped" => "stopped",
                "FinishedTrack" => "finished",
                _ => "unknown",
            },
        };
        println!("{}|||", state);
    }
}

pub fn run_monitor(socket_path: &PathBuf) {
    eprintln!("Starting monitor mode...");
    eprintln!("Using socket path: {}", socket_path.display());

    loop {
        // Wait for socket to be available
        if let Err(e) = wait_for_socket(socket_path.to_str().unwrap()) {
            eprintln!("Error waiting for socket: {}", e);
            std::process::exit(1);
        }

        // Connect to ncspot's socket
        let stream = match UnixStream::connect(&socket_path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to connect to socket: {}", e);
                eprintln!("Retrying...");
                continue;
            }
        };

        eprintln!("Connected to ncspot socket, waiting for messages...");

        let reader = BufReader::new(stream);
        for line in reader.lines() {
            match line {
                Ok(content) => match serde_json::from_str::<NcspotEvent>(&content) {
                    Ok(event) => handle_event(event),
                    Err(e) => {
                        eprintln!("Failed to parse JSON: {}", e);
                        eprintln!("Raw content: {}", content);
                    }
                },
                Err(err) => {
                    eprintln!("Connection lost: {}", err);
                    eprintln!("Waiting for ncspot to restart...");
                    break;
                }
            }
        }
    }
}
