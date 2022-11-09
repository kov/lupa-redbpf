use rocket::get;
use rust_embed::RustEmbed;
use std::borrow::Cow;
use std::path::Path;
use std::path::PathBuf;

#[derive(RustEmbed)]
#[folder = "$OUT_DIR/web/"]
struct Asset;

#[get("/")]
pub fn index() -> &'static [u8] {
    get_embedded_path("index.html").expect("index.html missing from the bundle")
}

#[get("/<path..>")]
pub fn embedded(path: PathBuf) -> &'static [u8] {
    get_embedded_path(&path).unwrap_or(&[])
}

fn get_embedded_path<P: AsRef<Path>>(path: P) -> Option<&'static [u8]> {
    match Asset::get(&path.as_ref().to_string_lossy()) {
        Some(content) => match content.data {
            Cow::Borrowed(bytes) => Some(bytes),
            Cow::Owned(_) => unreachable!(),
        },
        None => None,
    }
}
