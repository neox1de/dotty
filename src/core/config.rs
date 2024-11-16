use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Serialize, Deserialize)]
pub struct FileMapping {
    pub source: String,
    pub destination: String,
}

#[derive(Serialize, Deserialize)]
pub struct Package {
    pub name: String,
    #[serde(default)]
    pub is_aur: bool,
    #[serde(default)]
    pub files: Vec<FileMapping>,
}

#[derive(Serialize, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub skip_existing: bool,
}

#[derive(Serialize, Deserialize)]
pub struct DottyConfig {
    #[serde(default)]
    pub packages: Vec<Package>,
    #[serde(default)]
    pub settings: Option<Settings>,
}

impl DottyConfig {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let config_path = path.as_ref().join("dotty.yaml");
        
        if !config_path.exists() {
            anyhow::bail!("dotty.yaml not found in repository");
        }

        let contents = std::fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read {}", config_path.display()))?;

        serde_yaml::from_str(&contents)
            .with_context(|| format!("Failed to parse {} as YAML", config_path.display()))
    }

    pub fn validate(&self) -> Result<()> {
        // Validate packages if any exist
        if !self.packages.is_empty() {
            for package in &self.packages {
                if package.name.is_empty() {
                    anyhow::bail!("Package name cannot be empty");
                }
                
                if !package.files.is_empty() {
                    for file in &package.files {
                        if file.source.is_empty() {
                            anyhow::bail!("Source path cannot be empty for package {}", package.name);
                        }
                        if file.destination.is_empty() {
                            anyhow::bail!("Destination path cannot be empty for package {}", package.name);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub fn get_all_file_mappings(&self) -> Vec<&FileMapping> {
        self.packages
            .iter()
            .flat_map(|package| package.files.iter())
            .collect()
    }

    pub fn should_skip_existing(&self) -> bool {
        self.settings.as_ref().map_or(false, |s| s.skip_existing)
    }
} 