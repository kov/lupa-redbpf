#![feature(decl_macro)]
use anyhow::{bail, Result};
use rustyline::{self, error};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex},
};
use structopt::StructOpt;
use tracer::Event as TraceEvent;
use tracing::{trace, Level};
use tracing_subscriber::FmtSubscriber;

use crate::tracer::Tracer;

pub mod probe_serde;
mod tracer;

fn get_history_path() -> PathBuf {
    let mut config_dir = dirs::config_dir().unwrap();
    config_dir.push("lupa");

    std::fs::create_dir_all(&config_dir).unwrap();

    config_dir.push("history.txt");
    config_dir
}

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
    let open_files = Arc::new(Mutex::new(OpenFilesMap::new()));
    let event_history = Arc::new(Mutex::new(EventHistory::new()));

    // Run trace handling on a separate thread.
    let t_open_files = open_files.clone();
    let t_event_history = event_history.clone();
    let _tracer_thread = std::thread::spawn(move || {
        let tracer = Tracer::new(pid);
        for event in tracer {
            println!("{:#?}", event);
            match &event {
                TraceEvent::FileOpen { pid, fd, path } => {
                    t_open_files.lock().unwrap().insert(
                        (*pid, *fd),
                        File {
                            pid: *pid,
                            fd: *fd,
                            path: path.clone(),
                        },
                    );
                }
                TraceEvent::FileClose { pid, fd } => {
                    t_open_files.lock().unwrap().remove(&(*pid, *fd));
                }
                TraceEvent::FileOpenFail { .. } => {
                    trace!("Failed to open file");
                }
                TraceEvent::ProcessFailed { error } => bail!(error.clone()),
            };

            t_event_history.lock().unwrap().push(event);
        }
        Ok(())
    });

    // And handle command line on the main thread.
    let mut rl = rustyline::DefaultEditor::new()?;

    let history_path = get_history_path();

    if rl.load_history(&history_path).is_err() {
        println!("No previous history.");
    }

    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                println!("Line: {}", line);
            }
            Err(error::ReadlineError::Interrupted) => {
                println!("CTRL-C");
            }
            Err(error::ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }

    rl.save_history(&history_path).unwrap_or(());

    Ok(())
}
