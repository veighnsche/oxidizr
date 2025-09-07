#[derive(Debug, Clone)]
pub struct Command {
    pub command: String,
    pub args: Vec<String>,
}

impl Command {
    pub fn build(cmd: &str, args: &[&str]) -> Self {
        Self {
            command: cmd.to_string(),
            args: args.iter().map(|s| s.to_string()).collect(),
        }
    }

    pub fn command(&self) -> String {
        // Pre-allocate capacity to avoid repeated allocations
        let capacity = self.command.len() + self.args.iter().map(|a| a.len() + 1).sum::<usize>();
        let mut s = String::with_capacity(capacity);
        s.push_str(&self.command);
        for a in &self.args {
            s.push(' ');
            s.push_str(a);
        }
        s
    }
}
