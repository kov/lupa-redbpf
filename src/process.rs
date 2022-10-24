use std::{collections::HashMap, ffi::OsStr, os::unix::prelude::OsStrExt, path::PathBuf};

use probes::filetracker::{EventKind, FileEvent};
use tracing::trace;

pub(crate) struct Process {
    pub(crate) pid: u64,
    pub(crate) files: HashMap<u64, File>,
}

pub(crate) struct File {
    pub(crate) fd: u64,
    pub(crate) path: PathBuf,
}

impl Process {
    pub(crate) fn new(pid: u64) -> Self {
        Self {
            pid,
            files: HashMap::new(),
        }
    }

    pub(crate) fn handle_event(&mut self, event: &FileEvent) {
        // Ignore ourselves.
        if event.pid == std::process::id().into() {
            return;
        }

        // The openat kretprobe will send both successful and failed calls.
        // Filter out the failures, for now.
        if event.fd < 1 {
            trace!(
                "Event for failed open: {} => {}",
                event.pid,
                String::from_utf8(event.path.to_vec()).unwrap()
            );
            return;
        }

        let path_str = std::str::from_utf8(&event.path).expect("Failed UTF-8 conversion");
        trace!(
            "File event from PID {} fd {} path {}",
            event.pid,
            event.fd,
            path_str,
        );

        match event.kind {
            EventKind::Open => {
                let existing = self.files.insert(
                    event.fd as u64,
                    File {
                        fd: event.fd as u64,
                        path: PathBuf::from(OsStr::from_bytes(&event.path)),
                    },
                );

                // Note that this is very common on multithreaded applications as a close() syscall
                // will get the fd released kernel side before it returns, and its completion may be
                // preempted by an openat2() that gets that same fd number.
                if let Some(existing) = existing {
                    trace!(
                        "Duplicate file descriptor {} for PID {} path {} <=> {}",
                        existing.fd,
                        self.pid,
                        path_str,
                        existing.path.to_string_lossy()
                    );
                }
            }
            EventKind::Close => {
                let found = self.files.remove(&(event.fd as u64));
                if found.is_none() {
                    trace!(
                        "PID {} tried to close file descriptor {}, which we did not know",
                        self.pid,
                        event.fd
                    );
                }
            }
        }
    }
}
