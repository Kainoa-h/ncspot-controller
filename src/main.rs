mod ncspot_event;

use ncspot_event::{Mode, NcspotEvent};
use std::io::{BufRead, BufReader};
use std::os::unix::net::UnixStream;

fn main() {
    let socket_path = "/tmp/ncspot-501/ncspot.sock";

    // Connect to ncspot's socket
    let stream = UnixStream::connect(socket_path).expect("Failed to connect to ncspot socket");

    println!("Connected to ncspot socket, waiting for messages...");

    let reader = BufReader::new(stream);
    for line in reader.lines() {
        match line {
            Ok(content) => {
                match serde_json::from_str::<NcspotEvent>(&content) {
                    Ok(event) => {
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
                            event.playable.artists.join(", "),
                            event.playable.title,
                            event.playable.album
                        );
                    }
                    Err(e) => {
                        eprintln!("Failed to parse JSON: {}", e);
                        eprintln!("Raw content: {}", content);
                    }
                }
            }
            Err(err) => {
                eprintln!("Error reading line: {}", err);
                break;
            }
        }
    }
}
