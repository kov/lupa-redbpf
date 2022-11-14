use signal_hook::consts::SIGUSR1;
use std::{
    os::unix::process::CommandExt,
    process::Command,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

fn main() -> Result<(), std::io::Error> {
    let should_continue = Arc::new(AtomicUsize::new(0));

    signal_hook::flag::register_usize(SIGUSR1, Arc::clone(&should_continue), 1usize)?;

    'infinite: loop {
        match should_continue.load(Ordering::Relaxed) {
            0 => std::thread::sleep(Duration::from_millis(100)),
            1 => break 'infinite,
            _ => unreachable!(),
        }
    }

    let args: Vec<String> = std::env::args().collect();
    Err(Command::new(args[1].clone()).args(&args[2..]).exec())
}
