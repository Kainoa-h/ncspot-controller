use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc::channel;

pub fn get_ncspot_socket_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let output = Command::new("ncspot").arg("info").output()?;

    if !output.status.success() {
        return Err("Failed to run 'ncspot info'".into());
    }

    let stdout = String::from_utf8(output.stdout)?;

    for line in stdout.lines() {
        if line.starts_with("USER_RUNTIME_PATH") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let runtime_path = parts[1];
                return Ok(PathBuf::from(runtime_path).join("ncspot.sock"));
            }
        }
    }

    Err("Could not find USER_RUNTIME_PATH in ncspot info output".into())
}

pub fn wait_for_socket(socket_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new(socket_path);

    // If socket already exists, return immediately
    if path.exists() {
        return Ok(());
    }

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

pub fn send_command(socket_path: &PathBuf, command: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = UnixStream::connect(socket_path)?;
    writeln!(stream, "{}", command)?;
    stream.flush()?;

    // Read and discard the response (ncspot sends current state back)
    let mut response = String::new();
    let mut reader = BufReader::new(&stream);
    reader.read_line(&mut response)?;

    Ok(())
}
