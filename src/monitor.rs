use crate::config::Config;
use crate::ncspot_event::{Mode, NcspotEvent};
use crate::socket::wait_for_socket;
use std::io::{BufRead, BufReader};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::Duration;

fn mode_to_state_string(mode: &Mode) -> &str {
    match mode {
        Mode::Playing { .. } => "playing",
        Mode::Paused { .. } => "paused",
        Mode::Simple(s) => match s.as_str() {
            "Stopped" => "stopped",
            "FinishedTrack" => "finished",
            _ => "unknown",
        },
    }
}

fn execute_hook(config: &Config, state: &str, artist: &str, title: &str, album: &str) {
    if let Some(hook_script) = &config.hook_script {
        let expanded_path = shellexpand::tilde(hook_script).to_string();

        // Delay playing state to prevent race condition when ncspot goes into finished then
        // playing state really quickly when a song ends and another starts
        if state == "playing" {
            thread::sleep(Duration::from_millis(500));
        }

        if let Err(e) = Command::new(&expanded_path)
            .env("NCSPOT_STATE", state)
            .env("NCSPOT_ARTIST", artist)
            .env("NCSPOT_TITLE", title)
            .env("NCSPOT_ALBUM", album)
            .spawn()
        {
            println!("Failed to execute hook script: {}", e);
        }
    }
}

fn handle_event(event: NcspotEvent, config: &Config) {
    let (state, artist, title, album) = if let Some(playable) = event.playable {
        let state = mode_to_state_string(&event.mode);
        (
            state,
            playable.artists.join(", "),
            playable.title,
            playable.album,
        )
    } else {
        let state = mode_to_state_string(&event.mode);
        (state, String::new(), String::new(), String::new())
    };

    execute_hook(config, state, &artist, &title, &album);
}

fn send_stopped_event(config: &Config) {
    execute_hook(config, "stopped", "", "", "");
}

pub fn run_monitor(socket_path: &PathBuf) {
    let config = Config::load();

    loop {
        // Wait for socket to be available
        if let Err(e) = wait_for_socket(socket_path.to_str().unwrap()) {
            println!("Error waiting for socket: {}", e);
            std::process::exit(1);
        }

        // Connect to ncspot's socket
        let stream = match UnixStream::connect(socket_path) {
            Ok(s) => s,
            Err(e) => {
                println!("Failed to connect to socket: {}", e);
                continue;
            }
        };

        let reader = BufReader::new(stream);
        for line in reader.lines() {
            match line {
                Ok(content) => match serde_json::from_str::<NcspotEvent>(&content) {
                    Ok(event) => handle_event(event, &config),
                    Err(e) => {
                        println!("Failed to parse JSON: {}", e);
                    }
                },
                Err(e) => {
                    println!("Connection lost: {}", e);
                    send_stopped_event(&config);
                    break;
                }
            }
        }

        // Connection closed cleanly
        println!("ncspot connection closed");
        send_stopped_event(&config);
    }
}
