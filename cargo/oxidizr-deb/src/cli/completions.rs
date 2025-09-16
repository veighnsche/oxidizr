use clap::CommandFactory;
use clap_complete::shells::{Bash, Zsh, Fish};

use crate::cli::args::{Cli, Shell};

pub fn emit(shell: Shell) -> Result<(), String> {
    let mut cmd = Cli::command();
    match shell {
        Shell::Bash => clap_complete::generate(Bash, &mut cmd, "oxidizr-deb", &mut std::io::stdout()),
        Shell::Zsh => clap_complete::generate(Zsh, &mut cmd, "oxidizr-deb", &mut std::io::stdout()),
        Shell::Fish => clap_complete::generate(Fish, &mut cmd, "oxidizr-deb", &mut std::io::stdout()),
    }
    Ok(())
}
