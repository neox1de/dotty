use std::path::PathBuf;

pub fn validate_repo_format(repo: &str) -> Result<String, String> {
    if repo.contains('/') && repo.split('/').count() == 2 {
        Ok(repo.to_string())
    } else {
        Err("Repository must be in the format 'username/repo'".to_string())
    }
}

pub fn validate_folder_path(path: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(path);
    if path.exists() {
        Ok(path)
    } else {
        Err(format!("Folder path '{}' does not exist", path.display()))
    }
} 