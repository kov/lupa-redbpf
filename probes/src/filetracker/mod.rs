// FIXME: this should be 4096, like in the kernel, but using the correct value causes the
// memset to be called to initialize the array, which is a no-no, how do we forbid memset?
pub const PATH_MAX: usize = 256;

#[repr(u64)]
pub enum EventKind {
    Open,
    Close,
}

#[repr(C)]
pub struct FileEvent {
    pub pid: u64,
    pub kind: EventKind,
    pub fd: i64,
    pub path: [u8; PATH_MAX],
}

impl FileEvent {
    pub fn for_pid(pid: u64) -> Self {
        FileEvent {
            pid,
            fd: 0,
            kind: EventKind::Close,
            path: [0; PATH_MAX],
        }
    }
}

#[repr(C, packed(1))]
pub struct TracepointCommonArgs {
    pub ctype: u16,
    pub flags: u8,
    pub preempt_count: u8,
    pub pid: i32,
}

// See /sys/kernel/debug/tracing/events/syscalls/sys_enter_close/format.
#[repr(C, packed(1))]
pub struct SysEnterCloseArgs {
    pub common: TracepointCommonArgs,
    pub sys_nr: i32,
    pad: u32,
    pub fd: u64,
}
