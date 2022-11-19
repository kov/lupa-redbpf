use gloo_net::http::Request;
use gloo_timers::callback::Interval;
use serde::Deserialize;
use yew::prelude::*;

#[derive(Deserialize)]
struct File {
    fd: u64,
    path: String,
}

#[function_component(App)]
fn app() -> Html {
    let files = use_state(|| vec![]);

    let tfiles = files.clone();
    use_effect(move || {
        let interval = Interval::new(500, move || {
            let files = tfiles.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let fetched: Vec<File> = Request::get("/api/files")
                    .send()
                    .await
                    .unwrap()
                    .json()
                    .await
                    .unwrap();
                files.set(fetched);
            });
        });

        move || {
            interval.cancel();
        }
    });

    let files_html = files
        .iter()
        .map(|file| {
            html! {
                <tr>
                    <td>{&file.path}</td><td>{file.fd}</td>
                </tr>
            }
        })
        .collect::<Html>();

    html! {
        <>
            <h1>{ "Open files" }</h1>
            <table id="files-table">
                <tr>
                    <th>{ "Path" }</th><th>{ "FD" }</th>
                </tr>
                { files_html }
            </table>
        </>
    }
}

fn main() {
    yew::start_app::<App>();
}
