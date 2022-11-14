use yew::prelude::*;

#[function_component(App)]
fn app() -> Html {
    let files = use_state(|| vec![]);

    let tfiles = files.clone();
    use_effect_with_deps(
        move || {
            wasm_bindgen_futures::spawn_local(async move {
                let fetched: Vec<File> = Request::get("/api/files")
                    .send()
                    .await
                    .unwrap()
                    .json()
                    .await
                    .unwrap();
                tfiles.set(fetched);
            });
            || ()
        },
        (),
    );

    let files_html = files
        .iter()
        .map(|file| {
            html! {
                <li>{file.path}</li><li>{file.fd}</li>
            }
        })
        .collect::<Html>();

    html! {
        <>
            <h1>{ "Open files" }</h1>
            <ul>
                <li>path</li><li>fd</li>
                 { files_html }
            </ul>
        </>
    }
}

fn main() {
    yew::start_app::<App>();
}
