use crate::lupa::Lupa;
use crate::process::ProcessState;
use rocket::get;
use rocket::http::ContentType;
use rocket::response::content::Html;
use rocket::response::Content;
use rocket::State;
use rust_embed::RustEmbed;
use std::borrow::Cow;
use std::path::Path;
use std::path::PathBuf;

#[derive(RustEmbed)]
#[folder = "$OUT_DIR/web/"]
struct Asset;

#[get("/")]
pub fn index(lupa: State<Lupa>) -> Html<&'static [u8]> {
    match lupa.process.get_state() {
        ProcessState::NotStarted => lupa.process.spawn(),
        ProcessState::Running => println!("Process is running..."),
        ProcessState::Ended => println!("Process ended"),
    };

    Html(get_embedded_path("index.html").expect("index.html missing from the bundle"))
}

#[get("/<path..>")]
pub fn embedded(path: PathBuf) -> Content<&'static [u8]> {
    get_embedded_path(&path)
        .and_then(
            |bytes| match path.extension().and_then(|ext| ext.to_str()) {
                Some("wasm") => Some(Content(ContentType::WASM, bytes)),
                Some("js") => Some(Content(ContentType::JavaScript, bytes)),
                Some("css") => Some(Content(ContentType::CSS, bytes)),
                Some("ico") => Some(Content(ContentType::Icon, bytes)),
                Some(_) | None => Some(Content(ContentType::Binary, bytes)),
            },
        )
        .unwrap_or_else(|| Content(ContentType::Binary, &[]))
}

fn get_embedded_path<P: AsRef<Path>>(path: P) -> Option<&'static [u8]> {
    let path = path.as_ref();
    match Asset::get(&path.to_string_lossy()) {
        Some(content) => match content.data {
            Cow::Borrowed(bytes) => Some(bytes),
            Cow::Owned(_) => unreachable!(),
        },
        None => None,
    }
}
