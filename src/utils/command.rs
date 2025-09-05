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
        let mut s = self.command.clone();
        for a in &self.args {
            s.push(' ');
            s.push_str(a);
        }
        s
    }
}
