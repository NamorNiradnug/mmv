use std::{fs::File, path::PathBuf};

use tempdir::TempDir;

pub fn generate_files(
    files: impl Iterator<Item = PathBuf>,
    directories: impl Iterator<Item = PathBuf>,
) -> std::io::Result<TempDir> {
    let root_tmp_dir = TempDir::new("mmv-test")?;
    for file_path in files {
        let target_path = root_tmp_dir.path().join(file_path);
        std::fs::create_dir_all(target_path.parent().expect("cannot get parent directory"))?;
        File::create(target_path)?;
    }
    for directory_path in directories {
        std::fs::create_dir_all(root_tmp_dir.path().join(directory_path))?;
    }
    Ok(root_tmp_dir)
}
