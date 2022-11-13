use crate::probe_serde::*;
use futures::StreamExt;
use probes::filetracker::FileEvent;
use redbpf::load::Loader;
use tracing::warn;

mod probe_serde;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let mut filetracker = Loader::load(filetracker_probe_code()).expect("error on Loader::load");
    for kprobe in filetracker.kprobes_mut() {
        kprobe
            .attach_kprobe(&kprobe.name(), 0)
            .unwrap_or_else(|e| panic!("error attaching probe {}: {:#?}", kprobe.name(), e))
    }

    for tracepoint in filetracker.tracepoints_mut() {
        tracepoint
            .attach_trace_point("syscalls", &tracepoint.name())
            .unwrap_or_else(|e| {
                panic!(
                    "error attaching syscalls tracepoint {}: {:#?}",
                    tracepoint.name(),
                    e
                )
            })
    }

    while let Some((map_name, events)) = filetracker.events.next().await {
        if map_name == "file_events" {
            for event in events {
                let file_event = unsafe { std::ptr::read(event.as_ptr() as *const FileEvent) };
                match serde_json::to_string(&FileProbeIPC(file_event)) {
                    Ok(s) => println!("{}", s),
                    Err(e) => warn!("Failed to serialize event: {}", e),
                };
            }
        }
    }
}

fn filetracker_probe_code() -> &'static [u8] {
    include_bytes!(concat!(
        env!("OUT_DIR"),
        "/bpf/programs/filetracker/filetracker.elf"
    ))
}
