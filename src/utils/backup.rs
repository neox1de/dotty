use std::path::{Path, PathBuf};
use anyhow::Result;
use chrono::Local;
use crate::utils::output::*;

pub struct BackupManager {
    backup_dir: PathBuf,
    #[allow(dead_code)]
    timestamp: String,
}

impl BackupManager {
    pub fn new() -> Result<Self> {
        let cache_dir = dirs::cache_dir()
            .ok_or_else(|| anyhow::anyhow!("Failed to get cache directory"))?;
        
        let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
        let backup_dir = cache_dir
            .join("dotty")
            .join("dotty_backups")
            .join(&timestamp);

        std::fs::create_dir_all(&backup_dir)?;

        Ok(Self {
            backup_dir,
            timestamp,
        })
    }

    pub fn backup_if_exists(&self, path: &Path) -> Result<bool> {
        if !path.exists() {
            return Ok(false);
        }

        let relative_path = path.strip_prefix(dirs::home_dir().unwrap())?;
        let backup_path = self.backup_dir.join(relative_path);

        if let Some(parent) = backup_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        if path.is_dir() {
            print_info(&format!("Creating backup of directory: {}", path.display()));
            copy_dir_all(path, &backup_path)?;
        } else {
            print_info(&format!("Creating backup of file: {}", path.display()));
            std::fs::copy(path, &backup_path)?;
        }

        print_success(&format!("Backup created in: {}", backup_path.display()));
        Ok(true)
    }

    pub fn get_backup_dir(&self) -> &Path {
        &self.backup_dir
    }

    pub fn clean_backups() -> Result<()> {
        let cache_dir = dirs::cache_dir()
            .ok_or_else(|| anyhow::anyhow!("Failed to get cache directory"))?;
        
        let backup_dir = cache_dir.join("dotty").join("dotty_backups");
        
        if !backup_dir.exists() {
            print_info("No backups found.");
            return Ok(());
        }

        print_section("Cleaning Backups");
        print_info(&format!("Removing backup directory: {}", backup_dir.display()));

        match std::fs::remove_dir_all(&backup_dir) {
            Ok(_) => {
                print_success("All backups cleaned successfully!");
                Ok(())
            },
            Err(e) => {
                print_error(&format!("Failed to clean backups: {}", e));
                Err(e.into())
            }
        }
    }
}

fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    if !dst.exists() {
        std::fs::create_dir_all(dst)?;
    }

    for entry in walkdir::WalkDir::new(src) {
        let entry = entry?;
        let path = entry.path();
        let relative = path.strip_prefix(src)?;
        let target = dst.join(relative);

        if path.is_dir() {
            std::fs::create_dir_all(&target)?;
        } else {
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::copy(path, target)?;
        }
    }

    Ok(())
} 