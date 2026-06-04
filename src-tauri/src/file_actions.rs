use std::path::Path;

pub fn open_file(path: &str) -> anyhow::Result<()> {
    anyhow::ensure!(Path::new(path).exists(), "file does not exist");
    open::that(path)?;
    Ok(())
}

pub fn reveal_in_folder(path: &str) -> anyhow::Result<()> {
    let parent = Path::new(path)
        .parent()
        .ok_or_else(|| anyhow::anyhow!("file has no parent directory"))?;
    anyhow::ensure!(parent.exists(), "parent directory does not exist");
    open::that(parent)?;
    Ok(())
}

pub fn open_folder(path: &str) -> anyhow::Result<()> {
    let target = Path::new(path);
    anyhow::ensure!(target.exists(), "folder does not exist");
    anyhow::ensure!(target.is_dir(), "path is not a directory");
    open::that(path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_folder_returns_error_for_nonexistent_path() {
        let result = open_folder("C:/nonexistent_folder_12345");
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("folder does not exist"), "expected 'folder does not exist' but got: {}", msg);
    }

    #[test]
    fn open_folder_returns_error_for_file_path() {
        // Use the current test executable as a file path
        let exe = std::env::current_exe().unwrap();
        let result = open_folder(exe.to_str().unwrap());
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("path is not a directory"), "expected 'path is not a directory' but got: {}", msg);
    }
}
