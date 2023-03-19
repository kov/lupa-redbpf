#![feature(decl_macro)]
use anyhow::{bail, Result};
use crossterm::{
    event::{self, Event, KeyCode},
    execute, style, terminal,
};
use std::{
    collections::HashMap,
    panic::{self, PanicInfo},
    path::PathBuf,
};
use structopt::StructOpt;
use tracer::Event as TraceEvent;
use tracing::{trace, Level};
use tracing_subscriber::FmtSubscriber;
use tui::{backend, backend::CrosstermBackend, Terminal};

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
    panic::set_hook(Box::new(|info| {
        panic_hook(info);
    }));

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
    let mut terminal = setup_terminal()?;

    let mut open_files = OpenFilesMap::new();
    let mut event_history = EventHistory::new();

    let tracer = Tracer::new(pid);
    for event in tracer {
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

    terminal.show_cursor()?;
    restore_terminal()
}

fn setup_terminal() -> Result<Terminal<impl backend::Backend>> {
    terminal::enable_raw_mode()?;

    let mut stdout = std::io::stdout();

    execute!(
        stdout,
        terminal::EnterAlternateScreen,
        event::EnableMouseCapture
    )?;

    let terminal = Terminal::new(CrosstermBackend::new(stdout))?;

    Ok(terminal)
}

fn restore_terminal() -> Result<()> {
    terminal::disable_raw_mode()?;

    execute!(
        std::io::stdout(),
        terminal::LeaveAlternateScreen,
        event::DisableMouseCapture
    )?;

    Ok(())
}

// Stolen froHEAVILY INSPIRED on https://github.com/Rigellute/spotify-tui/blob/93fd30fa55accc3df6f1c1548de28c5465b97074/src/main.rs#L95
fn panic_hook(info: &PanicInfo<'_>) {
    if cfg!(debug_assertions) {
        let location = info.location().unwrap();

        let msg = match info.payload().downcast_ref::<&'static str>() {
            Some(s) => *s,
            None => match info.payload().downcast_ref::<String>() {
                Some(s) => &s[..],
                None => "Box<Any>",
            },
        };

        let stacktrace: String = format!("{:?}", backtrace::Backtrace::new()).replace('\n', "\n\r");

        terminal::disable_raw_mode().unwrap();
        execute!(
            std::io::stdout(),
            terminal::LeaveAlternateScreen,
            style::Print(format!(
                "thread '<unnamed>' panicked at '{}', {}\n\r{}",
                msg, location, stacktrace
            )),
            event::DisableMouseCapture
        )
        .unwrap();
    }
}
