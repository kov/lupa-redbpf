use crate::process::Process;

pub struct Lupa {
    pub process: Process,
}

impl Lupa {
    pub fn new() -> Self {
        Lupa {
            process: Process::new(vec!["python".to_string()]),
        }
    }
}
