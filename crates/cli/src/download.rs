use anyhow::bail;
use anyhow::{Context, Result};
use rayon::prelude::*;
use reqwest::blocking::get;
use std::fs::create_dir_all;
use std::fs::write;
use std::io::{Cursor, Read};
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};

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

    let response = get(item.url).with_context(|| format!("Failed to fetch {}", item.url))?;
    let status = response.status();
    if !status.is_success() {
        bail!("HTTP {status} for {}", item.url);
    }

    let bytes = response.bytes()?;
    write(&target, &bytes)?;

    let size_mb = bytes.len() as f64 / 1024.0 / 1024.0;
    println!("  Downloaded ({size_mb:.2} MB)");
    Ok(())
}

fn download_recursive_vf(output_dir: &Path) -> Result<()> {
    let target = output_dir.join(RECURSIVE_OUTPUT);
    println!("Downloading Recursive VF");
    println!("  {RECURSIVE_OUTPUT}");

    let response =
        get(RECURSIVE_ZIP_URL).with_context(|| format!("Failed to fetch {RECURSIVE_ZIP_URL}"))?;
    let status = response.status();
    if !status.is_success() {
        bail!("HTTP {status} for {RECURSIVE_ZIP_URL}");
    }

    let bytes = response.bytes()?;
    let cursor = Cursor::new(bytes.as_ref());
    let mut archive = zip::ZipArchive::new(cursor).context("Failed to open zip archive")?;

    let mut file = archive
        .by_name(RECURSIVE_ZIP_PATH)
        .with_context(|| format!("File {RECURSIVE_ZIP_PATH} not found in zip"))?;

    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    write(&target, &buffer)?;

    let size_mb = buffer.len() as f64 / 1024.0 / 1024.0;
    println!("  Downloaded ({size_mb:.2} MB)");
    Ok(())
}

pub fn download(build_dir: &Path) -> Result<()> {
    create_dir_all(build_dir)?;
    println!("Downloading fonts to {}", build_dir.display());

    let failure_count = AtomicUsize::new(0);
    let total_count = DOWNLOADS.len() + 1;

    let all_items: Vec<Option<&DownloadItem>> = DOWNLOADS
        .iter()
        .map(Some)
        .chain(std::iter::once(None)) // None represents Recursive VF
        .collect();

    all_items.par_iter().for_each(|item| {
        let result = match item {
            Some(dl) => download_file(dl, build_dir),
            None => download_recursive_vf(build_dir),
        };
        if let Err(e) = result {
            let name = item.map_or("Recursive VF", |i| i.description);
            eprintln!("Error downloading {name}: {e:?}");
            failure_count.fetch_add(1, Ordering::Relaxed);
        }
    });

    let failures = failure_count.load(Ordering::Relaxed);
    let success_count = total_count - failures;

    println!("\nDownload Summary");
    println!("  Success: {success_count}");
    if failures > 0 {
        println!("  Failed:  {failures}");
        bail!("Some downloads failed");
    }

    println!("All files ready in {}/", build_dir.display());
    Ok(())
}
