use std::{
    fs::{create_dir_all, write},
    io::{Cursor, Read},
    iter::once,
    path::Path,
    sync::atomic::{AtomicUsize, Ordering},
};

use anyhow::{Context, Result, bail};
use rayon::prelude::*;
use reqwest::blocking::get;

use crate::config::{
    JETBRAINS_MONO_FILENAME, JETBRAINS_MONO_LICENSE_URL, JETBRAINS_MONO_ZIP_PATH,
    JETBRAINS_MONO_ZIP_URL, NOTO_CJK_LICENSE_URL, NOTO_CJK_VF_FILENAME, NOTO_CJK_VF_URL,
    RECURSIVE_LICENSE_URL, RECURSIVE_VF_FILENAME, RECURSIVE_ZIP_PATH, RECURSIVE_ZIP_URL,
};

struct DownloadItem {
    url: &'static str,
    output_name: &'static str,
    description: &'static str,
}

const DOWNLOADS: &[DownloadItem] = &[
    DownloadItem {
        url: NOTO_CJK_VF_URL,
        output_name: NOTO_CJK_VF_FILENAME,
        description: "Noto Sans Mono CJK JP (Variable)",
    },
    DownloadItem {
        url: NOTO_CJK_LICENSE_URL,
        output_name: "LICENSE-NotoSansCJK.txt",
        description: "Noto CJK License",
    },
    DownloadItem {
        url: RECURSIVE_LICENSE_URL,
        output_name: "LICENSE-Recursive.txt",
        description: "Recursive License (OFL)",
    },
    DownloadItem {
        url: JETBRAINS_MONO_LICENSE_URL,
        output_name: "LICENSE-JetBrainsMono.txt",
        description: "JetBrains Mono License (OFL)",
    },
];

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
    let target = output_dir.join(RECURSIVE_VF_FILENAME);
    println!("Downloading Recursive VF");
    println!("  {RECURSIVE_VF_FILENAME}");

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

fn download_jetbrains_mono(output_dir: &Path) -> Result<()> {
    let target = output_dir.join(JETBRAINS_MONO_FILENAME);
    println!("Downloading JetBrains Mono");
    println!("  {JETBRAINS_MONO_FILENAME}");

    let response = get(JETBRAINS_MONO_ZIP_URL)
        .with_context(|| format!("Failed to fetch {JETBRAINS_MONO_ZIP_URL}"))?;
    let status = response.status();
    if !status.is_success() {
        bail!("HTTP {status} for {JETBRAINS_MONO_ZIP_URL}");
    }

    let bytes = response.bytes()?;
    let cursor = Cursor::new(bytes.as_ref());
    let mut archive = zip::ZipArchive::new(cursor).context("Failed to open zip archive")?;

    let mut file = archive
        .by_name(JETBRAINS_MONO_ZIP_PATH)
        .with_context(|| format!("File {JETBRAINS_MONO_ZIP_PATH} not found in zip"))?;

    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    write(&target, &buffer)?;

    let size_mb = buffer.len() as f64 / 1024.0 / 1024.0;
    println!("  Downloaded ({size_mb:.2} MB)");
    Ok(())
}

/// Download task type for parallel processing.
enum DownloadTask<'a> {
    File(&'a DownloadItem),
    RecursiveVf,
    JetBrainsMono,
}

pub fn download(build_dir: &Path) -> Result<()> {
    create_dir_all(build_dir)?;
    println!("Downloading fonts to {}", build_dir.display());

    let failure_count = AtomicUsize::new(0);

    // Build list of all download tasks
    let tasks: Vec<DownloadTask> = DOWNLOADS
        .iter()
        .map(DownloadTask::File)
        .chain(once(DownloadTask::RecursiveVf))
        .chain(once(DownloadTask::JetBrainsMono))
        .collect();

    let total_count = tasks.len();

    tasks.par_iter().for_each(|task| {
        let result = match task {
            DownloadTask::File(dl) => download_file(dl, build_dir),
            DownloadTask::RecursiveVf => download_recursive_vf(build_dir),
            DownloadTask::JetBrainsMono => download_jetbrains_mono(build_dir),
        };
        if let Err(e) = result {
            let name = match task {
                DownloadTask::File(dl) => dl.description,
                DownloadTask::RecursiveVf => "Recursive VF",
                DownloadTask::JetBrainsMono => "JetBrains Mono",
            };
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
