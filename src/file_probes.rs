use crate::probe_serde::*;
use probes::filetracker::FileEvent;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::mpsc::SyncSender;

pub fn run(child_pid: u64, tx: SyncSender<FileEvent>) {
    // let mut path = std::env::current_exe().expect("Could not identify my own path");
    // path.set_file_name("lupa-probe");

    // println!("path: {}", path.to_string_lossy());
    let path = "lupa-probe";
    let mut child = Command::new(path)
        .arg(child_pid.to_string())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to run lupa-probe");

    let mut stdout = BufReader::new(child.stdout.take().unwrap());
    loop {
        match child.try_wait() {
            Ok(Some(_status)) => break,
            Ok(None) => (),
            Err(e) => panic!("Failed to wait on lupa-probe: {}", e),
        }

        let mut buf = String::new();
        if let Ok(_) = stdout.read_line(&mut buf) {
            let event = FileEventSerDe::deserialize(&mut serde_json::Deserializer::from_str(&buf))
                .expect("Failed to deserialize the file event");
            tx.send(FileEvent::from(event))
                .expect("Unable to send event through channel");
        }
    }
}
