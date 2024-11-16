use clap::Parser;
use std::path::PathBuf;
use super::banner::BANNER;
use crate::utils::validation::{validate_repo_format, validate_folder_path};
use colored::*;

#[derive(Parser, Debug)]
#[command(
    name = "dotty",
    version,
    about = format!("{}\n{}", 
        BANNER,
        "A modern dotfile manager for your archlinux setup".bright_cyan().bold()
    ),
    help_template = format!("{{before-help}}{{about}}\n\n{}\n{{options}}\n\n{}\n{{after-help}}", 
        "Commands:".bright_yellow().bold(),
        "Additional Information:".bright_yellow().bold(),
    ),
)]
pub struct Args {
    /// Clone and install from GitHub
    #[arg(
        short,
        long,
        value_name = "USERNAME/REPO",
        value_parser = validate_repo_format,
        help_heading = "Installation Options"
    )]
    pub repo: Option<String>,

    /// Install from local folder
    #[arg(
        short,
        long,
        value_name = "PATH",
        value_parser = validate_folder_path,
        help_heading = "Installation Options"
    )]
    pub folder: Option<PathBuf>,

    /// Clean backup files
    #[arg(
        short = 'c',
        long,
        help_heading = "Management Options",
        help = "Remove all backup files from ~/.cache/dotty/dotty_backups"
    )]
    pub clean: bool,
}

impl Args {
    pub fn show_help_if_no_args() {
        println!("{}", BANNER);
        println!("\n{}", "A modern dotfile manager for your archlinux setup".bright_cyan().bold());
        
        println!("\n{}", "Commands:".bright_yellow().bold());
        println!("  {} Clone and install from GitHub", "dotty -r username/repo".bright_white());
        println!("  {} Install from local folder", "dotty -f ~/.dotfiles".bright_white());
        println!("  {} Clean backup files", "dotty -c".bright_white());
        
        println!("\n{}", "Use dotty -h for more information".bright_blue());
    }

    pub fn show_help() {
        println!("{}", BANNER);
        println!("\n{}", "A modern dotfile manager for your archlinux setup".bright_cyan().bold());
        
        println!("\n{}", "Commands:".bright_yellow().bold());
        println!("  {} Clone and install from GitHub", "dotty -r, --repo <USERNAME/REPO>".bright_white());
        println!("  {} Install from local folder", "dotty -f, --folder <PATH>".bright_white());
        println!("  {} Clean backup files", "dotty -c, --clean".bright_white());

        println!("\n{}", "Examples:".bright_yellow().bold());
        println!("  {} Install dotfiles from GitHub:", "→".bright_blue());
        println!("    {}", "dotty -r h3li0p4us3/dotfiles".bright_white());
        
        println!("  {} Clean up old backups:", "→".bright_blue());
        println!("    {}", "dotty -c".bright_white());

        println!("\n{}", "Additional Information:".bright_yellow().bold());
        println!("  • Configuration is read from {}", "dotty.yaml".bright_white());
        println!("  • Backups are stored in {}", "~/.cache/dotty/dotty_backups".bright_white());
        println!("  • Version: {}", env!("CARGO_PKG_VERSION").bright_white());
    }
} 