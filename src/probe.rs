use crate::probe_serde::*;
use futures::StreamExt;
use probes::filetracker::{FileEvent, ProcessEvent};
use redbpf::{load::Loader, Array};
use tracing::warn;

mod probe_serde;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let hm = HashMap::new();
    let pid: u64 = std::env::args()
        .nth(1)
        .expect("Expected PID as first argument")
        .parse()
        .expect("First argument needs to be a valid PID number");

    let mut filetracker = Loader::load(filetracker_probe_code()).expect("error on Loader::load");

    Array::<u64>::new(
        filetracker
            .map_mut("pid_to_trace")
            .expect("Failed to obtain PID to track map from probe"),
    )
    .unwrap()
    .set(0, pid)
    .expect("Failed to set PID on the probe's map");

    for kprobe in filetracker.kprobes_mut() {
        kprobe
            .attach_kprobe(&kprobe.name(), 0)
            .unwrap_or_else(|e| panic!("error attaching probe {}: {:#?}", kprobe.name(), e))
    }

    for tracepoint in filetracker.tracepoints_mut() {
        let name = tracepoint.name();
        let category = if name.starts_with("sched_") {
            "sched"
        } else if name.starts_with("sys_") {
            "syscalls"
        } else {
            unreachable!()
        };

        tracepoint
            .attach_trace_point(category, &name)
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
        } else if map_name == "process_events" {
            for event in events {
                let process_event =
                    unsafe { std::ptr::read(event.as_ptr() as *const ProcessEvent) };
                match serde_json::to_string(&ProcessProbeIPC(process_event)) {
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
