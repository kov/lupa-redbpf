#![no_std]
#![no_main]
use cty::*;
use probes::filetracker::{EventKind, FileEvent, SysEnterCloseArgs, TracepointCommonArgs};
use redbpf_probes::kprobe::prelude::*;
use redbpf_probes::tracepoint::prelude::*;

program!(0xFFFFFFFE, "GPL");

#[map]
static mut file_events: PerfMap<FileEvent> = PerfMap::with_max_entries(1024);

#[kretprobe]
fn do_sys_openat2(regs: Registers, parms: [u64; 5]) {
    let pid_tgid = bpf_get_current_pid_tgid();

    let mut event = FileEvent::for_pid(pid_tgid >> 32);
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
    }

    unsafe { file_events.insert(regs.ctx, &event) };
}

#[tracepoint]
fn sys_enter_close(args: *const SysEnterCloseArgs) {
    let args = unsafe { &*args };
    let pid_tgid = bpf_get_current_pid_tgid();

    let mut event = FileEvent::for_pid(pid_tgid >> 32);
    event.fd = args.fd as i64;

    unsafe {
        file_events.insert(
            &args.common as *const TracepointCommonArgs as *mut TracepointCommonArgs,
            &event,
        )
    };
}
