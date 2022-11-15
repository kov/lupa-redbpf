use crate::file_probes;
use probes::filetracker::{EventKind, FileEvent};
use serde::Serialize;
use std::{
    collections::HashMap,
    path::PathBuf,
    process::{Child, ExitStatus},
    sync::{mpsc::sync_channel, Arc, RwLock},
    thread::JoinHandle,
};
use tracing::trace;

type FilesTracker = Arc<RwLock<HashMap<u64, File>>>;

struct RunningProcess {
    command: Vec<String>,
    child: Child,
    probe: JoinHandle<()>,
}

type StateDetail = Arc<RwLock<ProcessStateDetail>>;
enum ProcessStateDetail {
    NotStarted(Vec<String>),
    Running(RunningProcess),
    Ended(ExitStatus),
}

pub enum ProcessState {
    NotStarted,
    Running,
    Ended,
}

impl From<&ProcessStateDetail> for ProcessState {
    fn from(detail: &ProcessStateDetail) -> ProcessState {
        match detail {
            ProcessStateDetail::NotStarted(_) => ProcessState::NotStarted,
            ProcessStateDetail::Running(_) => ProcessState::Running,
            ProcessStateDetail::Ended(_) => ProcessState::Ended,
        }
    }
}

pub struct Process {
    pub files: FilesTracker,
    state: StateDetail,
}

#[derive(Clone, Debug, Serialize)]
pub struct File {
    pub fd: u64,
    pub path: PathBuf,
}

impl Process {
    pub fn new(command: Vec<String>) -> Self {
        Self {
            state: Arc::new(RwLock::new(ProcessStateDetail::NotStarted(command))),
            files: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn get_state(&self) -> ProcessState {
        ProcessState::from(&*self.state.read().expect("State lock was poisoned"))
    }

    pub fn spawn(&self) {
        let mut state = self.state.write().expect("State lock was poisoned");

        assert!(matches!(*state, ProcessStateDetail::NotStarted(_)));

        let (tx, rx) = sync_channel(4096);

        if let ProcessStateDetail::NotStarted(args) = &*state {
            let mut path = std::env::current_exe().expect("Could not identify my own path");
            path.set_file_name("lupa-wrapper");

            let child = std::process::Command::new(path)
                .args(args.as_slice())
                .spawn()
                .expect("Failed to launch process");

            let files = self.files.clone();
            let child_pid = child.id() as u64;
            std::thread::spawn(move || {
                while let Ok(event) = rx.recv() {
                    Process::handle_event(child_pid, &files, &event);
                }
            });

            let child_pid = child.id() as u64;
            let probe = std::thread::spawn(move || {
                file_probes::run(child_pid, tx);
            });

            *state = ProcessStateDetail::Running(RunningProcess {
                command: args.clone(),
                child,
                probe,
            });

            let tstate = self.state.clone();
            std::thread::spawn(move || {
                let mut new_state = None;

                while let None = new_state {
                    if let ProcessStateDetail::Running(state) =
                        &mut *tstate.write().expect("State lock was poisoned")
                    {
                        if let Some(status) = state
                            .child
                            .try_wait()
                            .expect("Failed to wait for child process")
                        {
                            new_state = Some(ProcessStateDetail::Ended(status));
                        }
                    }

                    std::thread::sleep(std::time::Duration::from_millis(100));
                }

                if let Some(new_state) = new_state {
                    *tstate.write().expect("State lock was poisoned") = new_state;
                }
            });
        }
    }

    pub fn handle_event(child_pid: u64, files: &FilesTracker, event: &FileEvent) {
        // Ignore ourselves.
        if event.pid == std::process::id() as u64 {
            return;
        }

        if child_pid != event.pid {
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

        let path_str = std::str::from_utf8(&event.path)
            .expect("Failed UTF-8 conversion")
            .trim_end_matches(char::from(0));
        trace!(
            "File event from PID {} fd {} path {}",
            event.pid,
            event.fd,
            path_str,
        );

        let mut files = files.write().expect("Files tracker lock was poisoned.");
        match event.kind {
            EventKind::Open => {
                let existing = files.insert(
                    event.fd as u64,
                    File {
                        fd: event.fd as u64,
                        path: PathBuf::from(&path_str),
                    },
                );

                // Note that this is very common on multithreaded applications as a close() syscall
                // will get the fd released kernel side before it returns, and its completion may be
                // preempted by an openat2() that gets that same fd number.
                if let Some(existing) = existing {
                    trace!(
                        "Duplicate file descriptor {} for PID {} path {} <=> {}",
                        existing.fd,
                        child_pid,
                        path_str,
                        existing.path.to_string_lossy()
                    );
                }
            }
            EventKind::Close => {
                let found = files.remove(&(event.fd as u64));
                if found.is_none() {
                    trace!(
                        "PID {} tried to close file descriptor {}, which we did not know",
                        child_pid,
                        event.fd
                    );
                }
            }
        }
    }
}
