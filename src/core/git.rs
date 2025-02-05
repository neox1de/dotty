use anyhow::{Context, Result};
use git2::Repository;
use std::path::PathBuf;

pub struct GitRepo {
    pub username: String,
    pub repo_name: String,
}

impl GitRepo {
    pub fn from_string(repo_string: &str) -> Result<Self> {
        let parts: Vec<&str> = repo_string.split('/').collect();
        if parts.len() != 2 {
            anyhow::bail!("Invalid repository format. Expected 'username/repo'");
        }

        Ok(Self {
            username: parts[0].to_string(),
            repo_name: parts[1].to_string(),
        })
    }

    pub fn clone_url(&self) -> String {
        format!("https://github.com/{}/{}.git", self.username, self.repo_name)
    }

    pub fn folder_name(&self) -> String {
        format!("{}_{}", self.username, self.repo_name)
    }

    pub fn clone_to(&self, base_path: PathBuf) -> Result<Repository> {
        let repo_path = base_path.join(self.folder_name());
        Repository::clone(&self.clone_url(), &repo_path)
            .with_context(|| format!("Failed to clone repository {}", self.clone_url()))
    }
} 