use std::process::{Command, Stdio};
use anyhow::{Context, Result};
use crate::utils::spinner::with_spinner;

pub fn ask_for_sudo_password() -> Result<()> {
    let status = Command::new("sudo")
        .arg("-v")
        .status()
        .context("Failed to prompt for sudo password")?;

    if !status.success() {
        anyhow::bail!("Failed to authenticate with sudo");
    }
    Ok(())
}

pub fn install_package(package: &str) -> Result<()> {
    with_spinner(&format!("Installing {}...", package), || {
        let status = Command::new("sudo")
            .args(["pacman", "-S", "--noconfirm", package])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .context("Failed to run pacman")?;

        if !status.success() {
            anyhow::bail!("Failed to install {}", package);
        }
        Ok(())
    })
}

pub fn install_packages(packages: &[&str]) -> Result<()> {
    ask_for_sudo_password()?;
    for package in packages {
        install_package(package)?;
    }
    Ok(())
} 