#![feature(decl_macro)]
use structopt::StructOpt;
use tracing::Level;
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

fn main() {
    let opt = Opt::from_args();

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::WARN)
        .finish();

    tracing::subscriber::set_global_default(subscriber).unwrap();

    if let Some(cmd) = opt.cmd {
        match cmd {
            Command::Trace { pid } => {
                println!("pid: {}", pid);

                let tracer = Tracer::new(pid);
                for event in tracer {
                    println!("{:#?}", event)
                }
            }
        }
    }
}
