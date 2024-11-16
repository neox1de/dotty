use std::path::{Path, PathBuf};
use anyhow::Result;
use shellexpand;
use walkdir::WalkDir;
use crate::utils::output::*;
use crate::utils::backup::BackupManager;
use crate::core::config::DottyConfig;

pub fn copy_files(repo_path: &Path, config: &DottyConfig) -> Result<()> {
    print_section("Copying Files");

    let backup_manager = BackupManager::new()?;
    let all_files = config.get_all_file_mappings();
    let total_mappings = all_files.len();
    let mut completed = 0;

    for mapping in all_files {
        completed += 1;
        let source_path = repo_path.join(mapping.source.trim_start_matches('/'));
        let expanded_dest = shellexpand::tilde(&mapping.destination).into_owned();
        let destination_path = PathBuf::from(expanded_dest);

        print_subsection(&format!("Processing {}/{}", completed, total_mappings));
        print_info(&format!("Source: {}", mapping.source));
        print_info(&format!("Destination: {}", mapping.destination));

        if !source_path.exists() {
            print_error(&format!("Source path does not exist: {}", source_path.display()));
            continue;
        }

        if destination_path.exists() {
            match backup_manager.backup_if_exists(&destination_path) {
                Ok(true) => print_success("Backup created successfully"),
                Ok(false) => print_info("No backup needed"),
                Err(e) => {
                    print_error(&format!("Failed to create backup: {}", e));
                    continue;
                }
            }
        }

        if source_path.is_dir() {
            if let Err(e) = std::fs::create_dir_all(&destination_path) {
                print_error(&format!("Failed to create directory: {}", e));
                continue;
            }
            print_success(&format!("Created directory: {}", destination_path.display()));
            
            print_status("Copying contents...");
            copy_directory(&source_path, &destination_path, config.should_skip_existing())?;
        } else {
            if let Some(parent) = destination_path.parent() {
                if let Err(e) = std::fs::create_dir_all(parent) {
                    print_error(&format!("Failed to create parent directory: {}", e));
                    continue;
                }
            }
            copy_file(&source_path, &destination_path, config.should_skip_existing())?;
        }
    }

    print_separator();
    print_success("All files copied successfully!");
    print_info(&format!("Backups are stored in: {}", backup_manager.get_backup_dir().display()));
    Ok(())
}

fn copy_directory(source: &Path, destination: &Path, skip_existing: bool) -> Result<()> {
    for entry in WalkDir::new(source).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        let relative = path.strip_prefix(source)?;
        let target = destination.join(relative);

        if path.is_file() {
            if skip_existing && target.exists() {
                print_list_item(&format!("Skipped: {}", relative.display()));
                continue;
            }

            if let Some(parent) = target.parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent)?;
                }
            }

            match std::fs::copy(path, &target) {
                Ok(_) => print_list_item(&format!("Copied: {}", relative.display())),
                Err(e) => print_error(&format!("Failed to copy {}: {}", relative.display(), e)),
            }
        } else if path.is_dir() && !target.exists() {
            match std::fs::create_dir_all(&target) {
                Ok(_) => print_list_item(&format!("Created: {}", relative.display())),
                Err(e) => print_error(&format!("Failed to create directory {}: {}", relative.display(), e)),
            }
        }
    }

    Ok(())
}

fn copy_file(source: &Path, destination: &Path, skip_existing: bool) -> Result<()> {
    if skip_existing && destination.exists() {
        print_list_item(&format!("Skipped: {}", destination.display()));
        return Ok(());
    }

    match std::fs::copy(source, destination) {
        Ok(_) => {
            print_success(&format!("Copied: {} → {}", 
                source.display(), destination.display()));
            Ok(())
        },
        Err(e) => {
            print_error(&format!("Failed: {} → {}", 
                source.display(), destination.display()));
            print_list_item(&format!("Error: {}", e));
            Err(e.into())
        }
    }
} 
