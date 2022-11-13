use crate::lupa::Lupa;
use crate::process::ProcessState;
use rocket::get;
use rocket::State;
use rust_embed::RustEmbed;
use std::borrow::Cow;
use std::path::Path;
use std::path::PathBuf;

#[derive(RustEmbed)]
#[folder = "$OUT_DIR/web/"]
struct Asset;

#[get("/")]
pub fn index(lupa: State<Lupa>) -> String {
    match lupa.process.get_state() {
        ProcessState::NotStarted => lupa.process.spawn(),
        ProcessState::Running => println!("Process is running..."),
        ProcessState::Ended => println!("Process ended"),
    };
    format!("Files open: {:#?}", lupa.process.files.read().unwrap())

    //get_embedded_path("index.html").expect("index.html missing from the bundle")
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
