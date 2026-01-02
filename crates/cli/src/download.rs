use anyhow::{Context, Result};
use std::fs;
use std::io::{Cursor, Read};
use std::path::Path;

struct DownloadItem {
    url: &'static str,
    output_name: &'static str,
    description: &'static str,
}

const DOWNLOADS: &[DownloadItem] = &[
    DownloadItem {
        url: "https://raw.githubusercontent.com/notofonts/noto-cjk/f8d157532fbfaeda587e826d4cd5b21a49186f7c/Sans/Variable/TTF/Mono/NotoSansMonoCJKjp-VF.ttf",
        output_name: "NotoSansMonoCJKjp-VF.ttf",
        description: "Noto Sans Mono CJK JP (Variable)",
    },
    DownloadItem {
        url: "https://raw.githubusercontent.com/notofonts/noto-cjk/f8d157532fbfaeda587e826d4cd5b21a49186f7c/Sans/LICENSE",
        output_name: "LICENSE-NotoSansCJK.txt",
        description: "Noto CJK License",
    },
    DownloadItem {
        url: "https://raw.githubusercontent.com/arrowtype/recursive/refs/tags/v1.085/OFL.txt",
        output_name: "LICENSE-Recursive.txt",
        description: "Recursive License (OFL)",
    },
];

const RECURSIVE_ZIP_URL: &str =
    "https://github.com/arrowtype/recursive/releases/download/v1.085/ArrowType-Recursive-1.085.zip";
const RECURSIVE_ZIP_PATH: &str =
    "ArrowType-Recursive-1.085/Recursive_Desktop/Recursive_VF_1.085.ttf";
const RECURSIVE_OUTPUT: &str = "Recursive_VF_1.085.ttf";

fn download_file(item: &DownloadItem, output_dir: &Path) -> Result<()> {
    let target = output_dir.join(item.output_name);
    println!("Downloading {}", item.description);
    println!("  {}", item.output_name);

    let response = reqwest::blocking::get(item.url)
        .with_context(|| format!("Failed to fetch {}", item.url))?;
    let status = response.status();
    if !status.is_success() {
        anyhow::bail!("HTTP {} for {}", status, item.url);
    }

    let bytes = response.bytes()?;
    fs::write(&target, &bytes)?;

    let size_mb = bytes.len() as f64 / 1024.0 / 1024.0;
    println!("  Downloaded ({:.2} MB)", size_mb);
    Ok(())
}

fn download_recursive_vf(output_dir: &Path) -> Result<()> {
    let target = output_dir.join(RECURSIVE_OUTPUT);
    println!("Downloading Recursive VF");
    println!("  {}", RECURSIVE_OUTPUT);

    let response = reqwest::blocking::get(RECURSIVE_ZIP_URL)
        .with_context(|| format!("Failed to fetch {}", RECURSIVE_ZIP_URL))?;
    let status = response.status();
    if !status.is_success() {
        anyhow::bail!("HTTP {} for {}", status, RECURSIVE_ZIP_URL);
    }

    let bytes = response.bytes()?;
    let cursor = Cursor::new(bytes.as_ref());
    let mut archive = zip::ZipArchive::new(cursor).context("Failed to open zip archive")?;

    let mut file = archive
        .by_name(RECURSIVE_ZIP_PATH)
        .with_context(|| format!("File {} not found in zip", RECURSIVE_ZIP_PATH))?;

    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    fs::write(&target, &buffer)?;

    let size_mb = buffer.len() as f64 / 1024.0 / 1024.0;
    println!("  Downloaded ({:.2} MB)", size_mb);
    Ok(())
}

pub fn download(build_dir: &Path) -> Result<()> {
    fs::create_dir_all(build_dir)?;
    println!("Downloading fonts to {}", build_dir.display());

    let mut failures = Vec::new();

    for item in DOWNLOADS {
        if let Err(e) = download_file(item, build_dir) {
            eprintln!("Error: {e:?}");
            failures.push(item.description);
        }
    }

    if let Err(e) = download_recursive_vf(build_dir) {
        eprintln!("Error: {e:?}");
        failures.push("Recursive VF");
    }

    let success_count = DOWNLOADS.len() + 1 - failures.len();

    println!("\nDownload Summary");
    println!("  Success: {}", success_count);
    if !failures.is_empty() {
        println!("  Failed:  {}", failures.len());
        for name in &failures {
            println!("    - {}", name);
        }
        anyhow::bail!("Some downloads failed");
    }

    println!("All files ready in {}/", build_dir.display());
    Ok(())
}
