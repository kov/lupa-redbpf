use crate::process::Process;

pub struct Lupa {
    pub process: Process,
}

impl Lupa {
    pub fn new() -> Self {
        Lupa {
            process: Process::new(
                vec!["/usr/bin/code"]
                    .into_iter()
                    .map(|s| s.to_string())
                    .collect(),
            ),
        }
    }
}
