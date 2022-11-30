#![feature(decl_macro)]
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

mod file_probes;
mod lupa;
pub mod probe_serde;
mod process;

fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::WARN)
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();
}
