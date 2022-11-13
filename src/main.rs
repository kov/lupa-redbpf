#![feature(decl_macro)]
use crate::lupa::Lupa;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

mod file_probes;
mod lupa;
pub mod probe_serde;
mod process;

#[cfg(feature = "web-frontend")]
mod web_frontend;

fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::WARN)
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let server = rocket::ignite();

    #[cfg(feature = "web-frontend")]
    let server = server.mount(
        "/",
        rocket::routes![web_frontend::index, web_frontend::embedded],
    );

    server.manage(Lupa::new()).launch();
}
