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
