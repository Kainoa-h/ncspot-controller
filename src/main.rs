mod ncspot_event;

use ncspot_event::{Mode, NcspotEvent};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::io::{BufRead, BufReader};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::sync::mpsc::channel;

fn wait_for_socket(socket_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new(socket_path);

    // If socket already exists, return immediately
    if path.exists() {
        return Ok(());
    }

    println!(
        "Waiting for socket file to be created at {}...",
        socket_path
    );

    // Watch the parent directory for the socket file to be created
    let parent_dir = path.parent().ok_or("Invalid socket path")?;
    let socket_name = path.file_name().ok_or("Invalid socket path")?;

    let (tx, rx) = channel();

    let mut watcher = RecommendedWatcher::new(
        move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                let _ = tx.send(event);
            }
        },
        Config::default(),
    )?;

    watcher.watch(parent_dir, RecursiveMode::NonRecursive)?;

    // Wait for the socket file to be created
    loop {
        match rx.recv() {
            Ok(event) => {
                // Check all event kinds that might indicate file creation
                match event.kind {
                    EventKind::Create(_) | EventKind::Modify(_) | EventKind::Any => {
                        for event_path in &event.paths {
                            if event_path.file_name() == Some(socket_name) {
                                println!("Socket file detected!");
                                // Double-check it exists
                                if path.exists() {
                                    return Ok(());
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            Err(_) => {
                return Err("Watcher channel closed".into());
            }
        }
    }
}

fn handle_event(event: NcspotEvent) {
    if let Some(playable) = event.playable {
        let mode_str = match &event.mode {
            Mode::Playing { .. } => "Playing",
            Mode::Paused { paused } => {
                &format!("Paused at {}:{:02}", paused.secs / 60, paused.secs % 60)
            }
            Mode::Simple(s) => s.as_str(),
        };
        println!(
            "[{}] {} - {} ({})",
            mode_str,
            playable.artists.join(", "),
            playable.title,
            playable.album
        );
    } else {
        let mode_str = match &event.mode {
            Mode::Playing { .. } => "Playing",
            Mode::Paused { paused } => {
                &format!("Paused at {}:{:02}", paused.secs / 60, paused.secs % 60)
            }
            Mode::Simple(s) => s.as_str(),
        };
        println!("[{}] No track", mode_str);
    }
}

fn main() {
    let socket_path = "/tmp/ncspot-501/ncspot.sock";

    loop {
        // Wait for socket to be available
        if let Err(e) = wait_for_socket(socket_path) {
            eprintln!("Error waiting for socket: {}", e);
            std::process::exit(1);
        }

        // Connect to ncspot's socket
        let stream = match UnixStream::connect(socket_path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to connect to socket: {}", e);
                println!("Retrying...");
                continue;
            }
        };

        println!("Connected to ncspot socket, waiting for messages...");

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
                    println!("Waiting for ncspot to restart...");
                    break;
                }
            }
        }
    }
}
