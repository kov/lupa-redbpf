#![no_std]
#![no_main]
#![allow(non_camel_case_types)]
use cty::*;
use probes::filetracker::{
    EventKind, FileEvent, ProcessEvent, SchedProcessExitArgs, SysEnterCloseArgs,
    TracepointCommonArgs,
};
use redbpf_probes::kprobe::prelude::*;
use redbpf_probes::tracepoint::prelude::*;

program!(0xFFFFFFFE, "GPL");

#[map]
static mut process_events: PerfMap<ProcessEvent> = PerfMap::with_max_entries(1024);

#[map]
static mut file_events: PerfMap<FileEvent> = PerfMap::with_max_entries(1024);

#[map]
static mut pid_to_trace: Array<u64> = Array::with_max_entries(1);

const MAX_CHILDREN: u32 = 1024;

#[map]
static mut children_to_trace: Array<u64> = Array::with_max_entries(MAX_CHILDREN);

// Keep track of processes started by the PIDs we trace, and trace them as well.
#[kretprobe]
fn kernel_clone(regs: Registers, params: [u64; 5]) {
    let pid = bpf_get_current_pid_tgid() >> 32;
    if should_trace(pid) {
        let child_pid = regs.rc() as u64;
        unsafe {
            for index in 0..MAX_CHILDREN {
                if let Some(to_track) = children_to_trace.get_mut(index) {
                    if *to_track == 0 {
                        *to_track = child_pid;
                        break;
                    }
                }
            }

            process_events.insert(
                regs.ctx,
                &ProcessEvent {
                    pid,
                    kind: EventKind::Open,
                },
            );
        }
        printk!("Child started for PID %llu: %llu", pid, child_pid);
    }
}

fn maybe_remove_trace(pid: u64) -> bool {
    unsafe {
        if let Some(to_track) = pid_to_trace.get(0) {
            if *to_track == pid {
                printk!("PID %llu, that we track, has ended", pid);
                pid_to_trace.set(0, &0);
                return true;
            }
        }

        for index in 0..MAX_CHILDREN {
            if let Some(to_track) = children_to_trace.get_mut(index) {
                if *to_track == pid {
                    *to_track = 0;
                    return true;
                }
            }
        }
    }

    false
}

// Check if the process that exited is one we are tracing, and remove them
// from our list of PIDs to trace.
#[tracepoint]
fn sched_process_exit(args: *const SchedProcessExitArgs) {
    let pid = bpf_get_current_pid_tgid() >> 32;

    if maybe_remove_trace(pid) {
        unsafe {
            process_events.insert(
                &(&*args).common as *const TracepointCommonArgs as *mut TracepointCommonArgs,
                &ProcessEvent {
                    pid,
                    kind: EventKind::Close,
                },
            );
        }
    }
}

fn should_trace(pid: u64) -> bool {
    unsafe {
        for index in 0..MAX_CHILDREN {
            if let Some(child_pid) = children_to_trace.get(index) {
                if pid == *child_pid {
                    return true;
                }
            }
        }

        let to_track = match pid_to_trace.get(0) {
            None => return false,
            Some(to_track) => *to_track,
        };

        pid == to_track
    }
}

#[kretprobe]
fn do_sys_openat2(regs: Registers, parms: [u64; 5]) {
    let pid = bpf_get_current_pid_tgid() >> 32;

    if !should_trace(pid) {
        return;
    }

    let mut event = FileEvent::for_pid(pid);
    event.fd = regs.rc() as i64;
    event.kind = EventKind::Open;

    unsafe {
        let path = parms[1] as *const u8;
        if bpf_probe_read_user_str(
            event.path.as_mut_ptr() as *mut _,
            event.path.len() as u32,
            path as *const _,
        ) <= 0
        {
            bpf_trace_printk(b"error on bpf_probe_read_user_str\0");
            return;
        }

        file_events.insert(regs.ctx, &event)
    };
}

#[tracepoint]
fn sys_enter_close(args: *const SysEnterCloseArgs) {
    let pid = bpf_get_current_pid_tgid() >> 32;

    if !should_trace(pid) {
        return;
    }

    let mut event = FileEvent::for_pid(pid);

    let args = unsafe { &*args };
    event.fd = args.fd as i64;

    unsafe {
        file_events.insert(
            &args.common as *const TracepointCommonArgs as *mut TracepointCommonArgs,
            &event,
        )
    };
}
