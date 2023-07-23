#![feature(decl_macro)]
use anyhow::{bail, Result};
use std::{collections::HashMap, path::PathBuf};
use structopt::StructOpt;
use tracer::Event as TraceEvent;
use tracing::{trace, Level};
use tracing_subscriber::FmtSubscriber;

use crate::tracer::Tracer;

pub mod probe_serde;
mod tracer;

#[derive(Debug, StructOpt)]
#[structopt(name = "lupa", about = "Watch or record files opened by a process.")]
struct Opt {
    #[structopt(subcommand)]
    cmd: Option<Command>,
}

#[derive(Debug, StructOpt)]
enum Command {
    Trace { pid: u64 },
}

fn main() -> Result<()> {
    let opt = Opt::from_args();

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::WARN)
        .finish();

    tracing::subscriber::set_global_default(subscriber).unwrap();

    if let Some(cmd) = opt.cmd {
        match cmd {
            Command::Trace { pid } => {
                println!("pid: {}", pid);
                return trace_pid(pid);
            }
        }
    }

    Ok(())
}

struct File {
    pid: u64,
    fd: u64,
    path: PathBuf,
}

type OpenFilesMap = HashMap<(u64, u64), File>;

type EventHistory = Vec<TraceEvent>;

fn trace_pid(pid: u64) -> Result<()> {
    let mut open_files = OpenFilesMap::new();
    let mut event_history = EventHistory::new();

    let tracer = Tracer::new(pid);
    for event in tracer {
        println!("{:#?}", event);
        match &event {
            TraceEvent::FileOpen { pid, fd, path } => {
                open_files.insert(
                    (*pid, *fd),
                    File {
                        pid: *pid,
                        fd: *fd,
                        path: path.clone(),
                    },
                );
            }
            TraceEvent::FileClose { pid, fd } => {
                open_files.remove(&(*pid, *fd));
            }
            TraceEvent::FileOpenFail { .. } => {
                trace!("Failed to open file");
            }
            TraceEvent::ProcessFailed { error } => bail!(error.clone()),
        };

        event_history.push(event);
    }

    Ok(())
}
