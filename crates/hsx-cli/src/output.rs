//! Terminal output formatting helpers.

use colored::Colorize;

/// Print a styled header.
pub fn header(text: &str) {
    println!("\n{}\n", text.bold().cyan());
}

/// Print a styled error.
pub fn error(text: &str) {
    eprintln!("{} {}", "error:".bold().red(), text);
}

/// Print a styled warning.
pub fn warning(text: &str) {
    eprintln!("{} {}", "warn:".bold().yellow(), text);
}

/// Print a styled info message.
pub fn info(text: &str) {
    println!("{} {}", "info:".bold().blue(), text);
}

/// Print a styled success message.
pub fn success(text: &str) {
    println!("{} {}", "ok:".bold().green(), text);
}
