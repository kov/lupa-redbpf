use crate::probe_serde::*;
use probes::filetracker::{EventKind, FileEvent as ProbeFileEvent};
use serde::Serialize;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::prelude::FileExt;
use std::process::{Command, Stdio};
use std::sync::mpsc::{Receiver, SyncSender};
use std::{path::PathBuf, sync::mpsc::sync_channel};
use tracing::trace;

pub struct Tracer {
    rx: Receiver<ProbeFileEvent>,
}

#[derive(Clone, Debug, Serialize)]
pub enum FileEvent {
    Open { pid: u64, fd: u64, path: PathBuf },
    OpenFail { pid: u64, errno: i64, path: PathBuf },
    Close { pid: u64, fd: u64 },
}

impl Tracer {
    pub fn new(pid: u64) -> Self {
        let (tx, rx) = sync_channel::<ProbeFileEvent>(4096);

        let child_pid = pid;
        std::thread::spawn(move || {
            run(child_pid, tx);
        });

        Self { rx }
    }
}

impl Iterator for Tracer {
    type Item = FileEvent;

    fn next(&mut self) -> Option<FileEvent> {
        if let Ok(event) = self.rx.recv() {
            let path_str = std::str::from_utf8(&event.path)
                .expect("Failed UTF-8 conversion")
                .trim_end_matches(char::from(0));

            trace!(
                "File {} event from PID {} fd {} path {}",
                if let EventKind::Open = event.kind {
                    "open"
                } else {
                    "close"
                },
                event.pid,
                event.fd,
                path_str,
            );

            Some(match event.kind {
                EventKind::Open => {
                    let path = PathBuf::from(path_str);

                    if event.fd < 0 {
                        FileEvent::OpenFail {
                            pid: event.pid,
                            errno: event.fd,
                            path,
                        }
                    } else {
                        FileEvent::Open {
                            pid: event.pid,
                            fd: event.fd as u64,
                            path,
                        }
                    }
                }
                EventKind::Close => FileEvent::Close {
                    pid: event.pid,
                    fd: event.fd as u64,
                },
            })
        } else {
            None
        }
    }
}

fn run(child_pid: u64, tx: SyncSender<ProbeFileEvent>) {
    let mut path: PathBuf;

    if let Ok(_) = std::env::var("LUPA_SYSTEM_PROBE") {
        path = PathBuf::from("lupa-probe");
    } else {
        path = std::env::current_exe().expect("Could not identify my own path");
        path.set_file_name("lupa-probe");
    }

    let mut child = Command::new(path)
        .arg(child_pid.to_string())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to run lupa-probe");

    let mut stdout = BufReader::new(child.stdout.take().unwrap());
    let mut stderr = BufReader::new(child.stderr.take().unwrap());
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let mut buf = String::new();
                while let Ok(bytes_read) = stderr.read_line(&mut buf) {
                    if bytes_read == 0 {
                        break;
                    }
                }

                if !status.success() {
                    panic!("{}", buf);
                }

                break;
            }
            Ok(None) => (),
            Err(e) => panic!("Failed to wait on lupa-probe: {}", e),
        }

        let mut buf = String::new();
        if let Ok(_) = stdout.read_line(&mut buf) {
            log_to_file(format!("line: {}", buf));
            if buf.is_empty() {
                continue;
            }

            let event = FileEventSerDe::deserialize(&mut serde_json::Deserializer::from_str(&buf))
                .expect("Failed to deserialize the file event");
            tx.send(ProbeFileEvent::from(event))
                .expect("Unable to send event through channel");
        }
    }
}

fn log_to_file<S: AsRef<str>>(m: S) {
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open("/tmp/log")
        .expect("Failed to create temporary log file");
    file.write_all(m.as_ref().as_bytes());
}
