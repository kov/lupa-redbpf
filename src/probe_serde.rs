use probes::filetracker::{EventKind, FileEvent, ProcessEvent, PATH_MAX};
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

#[derive(Serialize, Deserialize)]
#[serde(remote = "EventKind")]
pub enum EventKindSerDe {
    Open,
    Close,
}

#[derive(Serialize, Deserialize)]
#[serde(remote = "FileEvent")]
pub struct FileEventSerDe {
    pub pid: u64,
    #[serde(with = "EventKindSerDe")]
    pub kind: EventKind,
    pub fd: i64,
    #[serde(with = "BigArray")]
    pub path: [u8; PATH_MAX],
}

impl From<FileEventSerDe> for FileEvent {
    fn from(event: FileEventSerDe) -> FileEvent {
        FileEvent {
            pid: event.pid,
            kind: event.kind,
            fd: event.fd,
            path: event.path,
        }
    }
}

impl From<FileEvent> for FileEventSerDe {
    fn from(event: FileEvent) -> FileEventSerDe {
        FileEventSerDe {
            pid: event.pid,
            kind: event.kind,
            fd: event.fd,
            path: event.path,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct FileProbeIPC(#[serde(with = "FileEventSerDe")] pub FileEvent);

#[derive(Serialize, Deserialize)]
#[serde(remote = "ProcessEvent")]
pub struct ProcessEventSerDe {
    pub pid: u64,
    #[serde(with = "EventKindSerDe")]
    pub kind: EventKind,
}

impl From<ProcessEventSerDe> for ProcessEvent {
    fn from(event: ProcessEventSerDe) -> ProcessEvent {
        ProcessEvent {
            pid: event.pid,
            kind: event.kind,
        }
    }
}

impl From<ProcessEvent> for ProcessEventSerDe {
    fn from(event: ProcessEvent) -> ProcessEventSerDe {
        ProcessEventSerDe {
            pid: event.pid,
            kind: event.kind,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ProcessProbeIPC(#[serde(with = "ProcessEventSerDe")] pub ProcessEvent);
