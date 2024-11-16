use std::process::{Command, Stdio};
use anyhow::{Context, Result};
use crate::utils::{
    system::AURHelper,
    output::print_status,
    spinner::with_spinner,
};
use tempfile::TempDir;

pub struct AURInstaller {
    temp_dir: TempDir,
}

impl AURInstaller {
    pub fn new() -> Result<Self> {
        let temp_dir = TempDir::new()
            .context("Failed to create temporary directory")?;
        Ok(Self { temp_dir })
    }

    pub fn install_base_devel(&self) -> Result<()> {
        let status = Command::new("sudo")
            .args(["pacman", "-S", "--needed", "--noconfirm", "base-devel"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .context("Failed to run pacman")?;

        if !status.success() {
            anyhow::bail!("Failed to install base-devel");
        }
        Ok(())
    }

    pub fn clone_and_install(&self, package: &str, url: &str) -> Result<()> {
        let pkg_path = self.temp_dir.path().join(package);

        // Clone the repository
        Command::new("git")
            .args(["clone", url])
            .arg(&pkg_path)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .output()
            .context("Failed to clone repository")?;

        // Change to package directory
        std::env::set_current_dir(&pkg_path)
            .context("Failed to change directory")?;

        print_status("Installing package...");
        // Run makepkg
        let status = Command::new("makepkg")
            .args(["-si", "--noconfirm"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .context("Failed to run makepkg")?;

        if !status.success() {
            anyhow::bail!("Failed to install package");
        }

        Ok(())
    }
}

pub fn install_aur_helper(package: &str) -> Result<()> {
    let urls = [
        ("paru", "https://aur.archlinux.org/paru.git"),
        ("yay", "https://aur.archlinux.org/yay.git"),
    ];

    let (_, url) = urls.iter()
        .find(|(name, _)| *name == package)
        .ok_or_else(|| anyhow::anyhow!("Unsupported AUR helper: {}", package))?;

    let installer = AURInstaller::new()?;
    
    // Install base-devel first
    installer.install_base_devel()?;

    // Clone and install the AUR helper
    installer.clone_and_install(package, url)?;

    Ok(())
}

pub fn install_aur_package(helper: AURHelper, package: &str) -> Result<()> {
    with_spinner(&format!("Installing {} (AUR)...", package), || {
        let helper_cmd = helper.command()
            .ok_or_else(|| anyhow::anyhow!("No AUR helper available"))?;

        let status = Command::new(helper_cmd)
            .args(["-S", "--needed", "--noconfirm", package])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .context("Failed to run AUR helper")?;

        if !status.success() {
            anyhow::bail!("Failed to install {}", package);
        }
        Ok(())
    })
}

pub fn install_aur_packages(helper: AURHelper, packages: &[&str]) -> Result<()> {
    for package in packages {
        install_aur_package(helper, package)?;
    }
    Ok(())
} 