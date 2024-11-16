use std::process::Command;
use std::path::PathBuf;
use std::io::{self, Write};
use crate::utils::{
    install::aur::install_aur_helper,
    output::print_status,
};

/// Represents available AUR helpers
#[derive(Debug, Clone, Copy)]
pub enum AURHelper {
    /// Paru AUR Helper
    Paru,
    /// Yay AUR helper
    Yay,
    /// None (No AUR helper found)
    None,
}

impl AURHelper {
    pub fn command(&self) -> Option<&str> {
        match self {
            AURHelper::Paru => Some("paru"),
            AURHelper::Yay => Some("yay"),
            AURHelper::None => None,
        }
    }

    /// Returns true if this is the preferred AUR helper (paru)
    pub fn is_preferred(&self) -> bool {
        matches!(self, AURHelper::Paru)
    }

    /// Returns a description of the AUR helper
    pub fn description(&self) -> &str {
        match self {
            AURHelper::Paru => "Paru (Feature-rich AUR helper written in Rust)",
            AURHelper::Yay => "Yay (Yet Another Yogurt - AUR Helper written in Go)",
            AURHelper::None => "None",
        }
    }
}

/// Prompts user to choose an AUR helper to install
pub fn prompt_aur_helper_installation() -> Option<AURHelper> {
    println!("\nNo AUR helper found. Choose one to install:");
    println!("1) {}", AURHelper::Paru.description());
    println!("2) {}", AURHelper::Yay.description());
    println!("3) Skip (but AUR packages won't be installed)");
    
    print!("\nEnter your choice [1-3]: ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    match input.trim() {
        "1" => {
            if let Err(e) = install_aur_helper("paru") {
                print_status(&format!("Failed to install paru: {}", e));
                return None;
            }
            Some(AURHelper::Paru)
        }
        "2" => {
            if let Err(e) = install_aur_helper("yay") {
                print_status(&format!("Failed to install yay: {}", e));
                return None;
            }
            Some(AURHelper::Yay)
        }
        _ => None,
    }
}

/// Detects the available AUR helper, preferring paru over yay
pub fn detect_aur_helper() -> AURHelper {
    // Always check for paru first as it's the preferred helper
    if has_command("paru") {
        return AURHelper::Paru;
    }

    // Fall back to yay if paru is not available
    if has_command("yay") {
        return AURHelper::Yay;
    }

    AURHelper::None
}

/// Checks if a command exists in the system
fn has_command(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

pub fn get_aur_helper_path() -> Option<PathBuf> {
    match detect_aur_helper() {
        AURHelper::None => None,
        helper => {
            let cmd = helper.command()?;
            Command::new("which")
                .arg(cmd)
                .output()
                .ok()
                .and_then(|output| {
                    if output.status.success() {
                        String::from_utf8(output.stdout)
                            .ok()
                            .map(|s| PathBuf::from(s.trim()))
                    } else {
                        None
                    }
                })
        }
    }
} 