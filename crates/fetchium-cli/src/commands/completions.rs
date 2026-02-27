//! `hsx completions` — shell completion script generation (PRD §42).

use clap::CommandFactory;
use clap_complete::{generate, Shell};

/// Generate shell completion scripts for the given shell.
pub fn run(shell: Shell) {
    let mut cmd = crate::cli::Cli::command();
    let name = cmd.get_name().to_string();
    generate(shell, &mut cmd, name, &mut std::io::stdout());
}
