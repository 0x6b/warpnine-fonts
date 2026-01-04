use std::{fs::remove_dir_all, path::Path};

use anyhow::Result;

pub fn clean(build_dir: &Path, dist_dir: &Path) -> Result<()> {
    let mut removed = 0;

    if build_dir.exists() {
        remove_dir_all(build_dir)?;
        println!("Removed {}", build_dir.display());
        removed += 1;
    } else {
        println!("Skipped {} (not found)", build_dir.display());
    }

    if dist_dir.exists() {
        remove_dir_all(dist_dir)?;
        println!("Removed {}", dist_dir.display());
        removed += 1;
    } else {
        println!("Skipped {} (not found)", dist_dir.display());
    }

    println!("Cleaned {removed} directories");
    Ok(())
}
