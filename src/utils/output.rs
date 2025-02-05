use colored::*;
use std::fmt::Display;

const PREFIX_STEP: &str = "→";
const PREFIX_SUCCESS: &str = "✓";
const PREFIX_ERROR: &str = "✗";
const PREFIX_INFO: &str = "•";
const INDENT: &str = "  ";

pub fn print_step<T: Display>(message: T) {
    println!("\n{} {}", PREFIX_STEP.bright_blue(), message);
}

pub fn print_success<T: Display>(message: T) {
    println!("{} {}", PREFIX_SUCCESS.bright_green(), message);
}

pub fn print_error<T: Display>(message: T) {
    println!("{} {}", PREFIX_ERROR.bright_red(), message);
}

pub fn print_status<T: Display>(message: T) {
    println!("{}", message);
}

pub fn print_info<T: Display>(message: T) {
    println!("{} {}", PREFIX_INFO.bright_cyan(), message);
}

pub fn print_list_item<T: Display>(message: T) {
    println!("{}{}• {}", INDENT, INDENT, message);
}

pub fn print_section<T: Display>(title: T) {
    println!("\n{} {}", "┌".bright_black(), title);
}

pub fn print_subsection<T: Display>(title: T) {
    println!("\n{} {}", "├".bright_black(), title);
}

pub fn print_separator() {
    println!("\n{}", "─".repeat(50).bright_black());
} 