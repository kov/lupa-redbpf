#![feature(decl_macro)]
#![feature(stmt_expr_attributes)]
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

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

    server.launch();
}
