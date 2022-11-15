use crate::{lupa::Lupa, process::File};
use rocket::{get, State};
use rocket_contrib::json::Json;

#[get("/files")]
pub fn get_files(lupa: State<Lupa>) -> Json<Vec<File>> {
    Json(
        lupa.process
            .files
            .read()
            .expect("Files lock was poisoned")
            .values()
            .map(|f| (*f).clone())
            .collect(),
    )
}
