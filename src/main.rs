use clap::Parser;
use dotty::{
    cli::Args,
    core::{GitRepo, config::DottyConfig},
    utils::{
        output::{print_step, print_success, print_error, print_status},
        spinner::with_spinner,
        system::{detect_aur_helper, AURHelper, prompt_aur_helper_installation},
        install::{pacman, aur},
        files::copy_files,
        backup::BackupManager,
    },
};
use std::{path::PathBuf, io::{self, Write}};

fn get_cache_dir() -> PathBuf {
    dirs::cache_dir()
        .expect("Failed to get cache directory")
        .join("dotty")
        .join("clone")
}

fn ensure_cache_dir() -> std::io::Result<PathBuf> {
    let cache_dir = get_cache_dir();
    std::fs::create_dir_all(&cache_dir)?;
    Ok(cache_dir)
}

fn display_packages_and_confirm(config: &DottyConfig) -> bool {
    let (aur_packages, normal_packages): (Vec<_>, Vec<_>) = config.packages
        .iter()
        .partition(|p| p.is_aur);

    println!();

    if !normal_packages.is_empty() {
        print_status("Packages to be installed from official repositories:");
        let packages: Vec<_> = normal_packages.iter().map(|p| &p.name).collect();
        for package in packages {
            print_status(&format!("    • {}", package));
        }
    }

    if !aur_packages.is_empty() {
        match detect_aur_helper() {
            AURHelper::None => {
                match prompt_aur_helper_installation() {
                    Some(helper) => {
                        println!();
                        print_status(&format!("Packages to be installed from AUR (using {}):", 
                            helper.command().unwrap()));
                        let packages: Vec<_> = aur_packages.iter().map(|p| &p.name).collect();
                        for package in packages {
                            print_status(&format!("    • {}", package));
                        }
                    }
                    None => {
                        println!();
                        print_error("AUR helper required");
                        print_status("The following packages need to be installed from AUR:");
                        let packages: Vec<_> = aur_packages.iter().map(|p| &p.name).collect();
                        for package in packages {
                            print_status(&format!("    • {}", package));
                        }
                        println!();
                        print_status("Please install either paru or yay and try again.");
                        std::process::exit(1);
                    }
                }
            }
            helper => {
                println!();
                print_status(&format!("Packages to be installed from AUR (using {}):", 
                    helper.command().unwrap()));
                let packages: Vec<_> = aur_packages.iter().map(|p| &p.name).collect();
                for package in packages {
                    print_status(&format!("    • {}", package));
                }
            }
        }
    }

    println!();
    print!("Do you want to proceed with the installation? [Y/n] ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    
    let input = input.trim().to_lowercase();
    println!();
    !matches!(input.as_str(), "n" | "no")
}

fn install_packages(config: &DottyConfig) -> Result<(), anyhow::Error> {
    let (aur_packages, normal_packages): (Vec<_>, Vec<_>) = config.packages
        .iter()
        .partition(|p| p.is_aur);

    if !normal_packages.is_empty() {
        print_status("Installing packages from official repositories...");
        let packages: Vec<_> = normal_packages.iter().map(|p| p.name.as_str()).collect();
        pacman::install_packages(&packages)?;
    }

    if !aur_packages.is_empty() {
        let helper = detect_aur_helper();
        if let AURHelper::None = helper {
            anyhow::bail!("No AUR helper found for installing AUR packages");
        }

        println!();
        print_status("Installing packages from AUR...");
        let packages: Vec<_> = aur_packages.iter().map(|p| p.name.as_str()).collect();
        aur::install_aur_packages(helper, &packages)?;
    }

    println!();
    print_success("All packages installed successfully!");
    Ok(())
}
fn main() {
    // Parse command line arguments
    let args = if std::env::args().len() <= 1 {
        Args::show_help_if_no_args();
        std::process::exit(0);
    } else if std::env::args().any(|arg| arg == "-h" || arg == "--help") {
        Args::show_help();
        std::process::exit(0);
    } else {
        Args::parse()
    };

    // Handle cleanup if requested
    if args.clean {
        match BackupManager::clean_backups() {
            Ok(_) => std::process::exit(0),
            Err(e) => {
                print_error(&format!("Failed to clean backups: {}", e));
                std::process::exit(1);
            }
        }
    }

    // If no arguments provided, show help and exit
    if args.repo.is_none() && args.folder.is_none() {
        Args::show_help_if_no_args();
        return;
    }

    // Handle different command combinations
    match (&args.repo, &args.folder) {
        (Some(repo), None) => {
            print_step("Installing dotfiles");
            
            // Parse repository information
            let repo = match GitRepo::from_string(repo) {
                Ok(repo) => repo,
                Err(e) => {
                    print_error(&format!("Invalid repository format: {}", e));
                    std::process::exit(1);
                }
            };

            // Ensure cache directory exists
            let cache_dir = match ensure_cache_dir() {
                Ok(dir) => dir,
                Err(e) => {
                    print_error(&format!("Failed to create cache directory: {}", e));
                    std::process::exit(1);
                }
            };

            let repo_path = cache_dir.join(&repo.folder_name());
            
            // Check if repository already exists
            if repo_path.exists() {
                print_status(&format!(
                    "Repository already exists at {}",
                    repo_path.display()
                ));
            } else {
                // Clone the repository with a spinner
                let clone_result = with_spinner("Cloning repository...", || {
                    repo.clone_to(cache_dir.clone())
                });

                match clone_result {
                    Ok(_) => {
                        print_success(&format!(
                            "Successfully cloned {} to {}",
                            repo.clone_url(),
                            repo_path.display()
                        ));
                    }
                    Err(e) => {
                        print_error(&format!("Failed to clone repository: {}", e));
                        std::process::exit(1);
                    }
                }
            }

            // Check for dotty.yaml
            print_status("Checking for dotty.yaml configuration...");
            match DottyConfig::from_path(&repo_path) {
                Ok(config) => {
                    match config.validate() {
                        Ok(_) => {
                            print_success("Found valid dotty.yaml configuration");
                            
                            // Install packages if there are any
                            if !config.packages.is_empty() {
                                if !display_packages_and_confirm(&config) {
                                    print_status("Installation cancelled.");
                                    std::process::exit(0);
                                }
                                
                                if let Err(e) = install_packages(&config) {
                                    print_error(&format!("Failed to install packages: {}", e));
                                    std::process::exit(1);
                                }
                            }

                            // Copy dotfiles if any package has files
                            let all_files = config.get_all_file_mappings();
                            if !all_files.is_empty() {
                                println!();
                                print_status("Files to be copied:");
                                for mapping in &all_files {
                                    let source = repo_path.join(&mapping.source);
                                    let _expanded_dest = shellexpand::tilde(&mapping.destination);
                                    
                                    if source.is_dir() {
                                        print_status(&format!("    • Directory: {} → {}", 
                                            mapping.source, mapping.destination));
                                    } else {
                                        print_status(&format!("    • File: {} → {}", 
                                            mapping.source, mapping.destination));
                                    }
                                }

                                println!();
                                print!("Do you want to proceed with copying the files? [Y/n] ");
                                io::stdout().flush().unwrap();

                                let mut input = String::new();
                                io::stdin().read_line(&mut input).unwrap();
                                
                                let input = input.trim().to_lowercase();
                                println!();

                                if matches!(input.as_str(), "n" | "no") {
                                    print_status("File copying cancelled.");
                                    std::process::exit(0);
                                }

                                if let Err(e) = copy_files(&repo_path, &config) {
                                    print_error(&format!("Failed to copy files: {}", e));
                                    std::process::exit(1);
                                }
                            }

                            print_success("Dotfiles installation completed successfully!");
                        }
                        Err(e) => {
                            print_error(&format!("Invalid dotty.yaml configuration: {}", e));
                            std::process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    print_error(&format!("Failed to read dotty.yaml: {}", e));
                    print_status("You can create a dotty.yaml file manually or use the --init option to generate one.");
                    std::process::exit(1);
                }
            }
        }
        (None, Some(folder)) => {
            print_step("Installing dotfiles from local folder");
            
            // Verify folder exists and contains dotty.yaml
            let config_path = folder.join("dotty.yaml");
            if !config_path.exists() {
                print_error("No dotty.yaml found in the specified folder");
                print_status("Make sure the folder contains a valid dotty.yaml configuration file");
                std::process::exit(1);
            }

            // Parse and validate config
            print_status("Reading configuration...");
            match DottyConfig::from_path(folder) {
                Ok(config) => {
                    match config.validate() {
                        Ok(_) => {
                            print_success("Found valid dotty.yaml configuration");
                            
                            // Install packages if any
                            if !config.packages.is_empty() {
                                if !display_packages_and_confirm(&config) {
                                    print_status("Installation cancelled.");
                                    std::process::exit(0);
                                }
                                
                                if let Err(e) = install_packages(&config) {
                                    print_error(&format!("Failed to install packages: {}", e));
                                    std::process::exit(1);
                                }
                            }

                            // Copy dotfiles if any package has files
                            let all_files = config.get_all_file_mappings();
                            if !all_files.is_empty() {
                                println!();
                                print_status("Files to be copied:");
                                for mapping in &all_files {
                                    let source = folder.join(&mapping.source);
                                    let _expanded_dest = shellexpand::tilde(&mapping.destination);
                                    
                                    if source.is_dir() {
                                        print_status(&format!("    • Directory: {} → {}", 
                                            mapping.source, mapping.destination));
                                    } else {
                                        print_status(&format!("    • File: {} → {}", 
                                            mapping.source, mapping.destination));
                                    }
                                }

                                println!();
                                print!("Do you want to proceed with copying the files? [Y/n] ");
                                io::stdout().flush().unwrap();

                                let mut input = String::new();
                                io::stdin().read_line(&mut input).unwrap();
                                
                                let input = input.trim().to_lowercase();
                                println!();

                                if matches!(input.as_str(), "n" | "no") {
                                    print_status("File copying cancelled.");
                                    std::process::exit(0);
                                }

                                if let Err(e) = copy_files(folder, &config) {
                                    print_error(&format!("Failed to copy files: {}", e));
                                    std::process::exit(1);
                                }
                            }

                            print_success("Dotfiles installation completed successfully!");
                        }
                        Err(e) => {
                            print_error(&format!("Invalid dotty.yaml configuration: {}", e));
                            std::process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    print_error(&format!("Failed to read dotty.yaml: {}", e));
                    print_status("Make sure the folder contains a valid dotty.yaml configuration file");
                    std::process::exit(1);
                }
            }
        }
        _ => {
            print_error("Invalid combination of arguments. Please use only one of: --repo or --folder");
            Args::show_help_if_no_args();
            std::process::exit(1);
        }
    }
}

