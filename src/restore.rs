use crate::config::load_config;
use crate::db::{mysql_pool, test_connection};
use anyhow::{bail, Result};
use flate2::read::GzDecoder;
use mysql::prelude::Queryable;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use zip::ZipArchive;

#[derive(Default)]
struct RestoreStats {
    restored: usize,
    failed: usize,
}

pub fn handle(
    db: String,
    preset: Option<String>,
    backup: PathBuf,
    dry_run: bool,
    yes: bool,
) -> Result<()> {
    // Test connection
    test_connection()?;

    // 0. Check backup format
    let backup_folder = if backup.is_dir() {
        backup.clone()
    } else if backup.extension().map(|e| e == "zip").unwrap_or(false) {
        let out = backup.with_extension("");
        println!("📦 Detected ZIP archive: extracting to {}", out.display());
        extract_zip(&backup, &out)?;
        out
    } else {
        bail!("Unsupported backup format: {}", backup.extension().and_then(|e| e.to_str()).unwrap_or("<unknown>"));
    };

    // 1. Resolve exclude tables from preset
    let (exclude_tables, preset_name) = if let Some(name) = preset {
        let cfg = load_config()?;
        let preset = cfg.presets
            .get(&name)
            .ok_or_else(|| anyhow::anyhow!("Preset '{}' not found", name))?;

        (preset.exclude_tables.clone(), Some(name))
    } else {
        (Vec::new(), None)
    };

    // 2. Extract backup → Vec<PathBuf>
    let (sql_files, skipped) = extract_backup(&backup_folder, &exclude_tables)?;

    // 2,5. Dry run summary
    println!("========================================");
    println!("🗄 Database       : {}", db);
    println!("📂 Backup folder  : {}", backup_folder.display());
    println!("📝 Tables to restore: {}", sql_files.len());
    println!("⏭ Tables skipped   : {}", skipped);
    if let Some(name) = &preset_name {
        println!("🎛 Preset used     : {}", name);
    } else {
        println!("🎛 Preset used     : None");
    }
    println!("========================================");

    if dry_run {
        println!("⚠️ Dry run enabled — no changes will be made");
        return Ok(());
    }

    // Safety confirmation
    if !yes {
        use std::io::{self, Write};

        print!("Proceed with restore? (yes/no): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if input.trim() != "yes" {
            println!("❌ Aborted by user");
            return Ok(());
        }
    }

    // 3. Restore into MySQL
    let stats = restore_sql(&db, sql_files)?;

    // 4. Final summary
    println!();
    println!("========================================");
    println!("🎉 Restore completed!");
    println!("✅ Tables restored successfully: {}", stats.restored);
    println!("⏭ Tables skipped (preset/exclude) : {}", skipped);
    println!("❌ Tables failed to restore      : {}", stats.failed);
    if let Some(name) = &preset_name {
        println!("🎛 Preset used     : {}", name);
    }
    println!("========================================");

    Ok(())
}

fn extract_backup(
    backup: &Path,
    exclude: &[String],
) -> Result<(Vec<PathBuf>, usize)> {
    let mut extracted = Vec::new();
    let mut skipped = 0;

    for entry in fs::read_dir(backup)? {
        let path = entry?.path();

        if path.extension() != Some("gz".as_ref()) {
            continue;
        }

        let table = path
            .file_name()
            .and_then(|s| s.to_str())
            .and_then(|s| s.strip_suffix(".sql.gz"))
            .unwrap_or("");

        if exclude.iter().any(|e| e == table) {
            skipped += 1;
            continue;
        }

        let output = path.with_extension("");
        let mut decoder = GzDecoder::new(fs::File::open(&path)?);
        let mut sql = String::new();
        decoder.read_to_string(&mut sql)?;
        fs::write(&output, sql)?;
        extracted.push(output);
    }

    Ok((extracted, skipped))
}

use std::io::{self, Write};

fn restore_sql(db: &str, files: Vec<PathBuf>) -> Result<RestoreStats> {
    let pool = mysql_pool()?;
    let mut conn = pool.get_conn()?;

    // 0. Check if database already exists
    let db_exists: Option<String> = conn
        .query_first(format!(
            "SELECT SCHEMA_NAME FROM INFORMATION_SCHEMA.SCHEMATA WHERE SCHEMA_NAME = '{}'",
            db
        ))?;

    if db_exists.is_some() {
        println!("⚠️ Database '{}' already exists!", db);

        print!("Do you want to restore into the existing database? (yes/no): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if input.trim() != "yes" {
            println!("❌ Aborted by user");
            return Ok(RestoreStats::default());
        }
    }

    // 1. Ensure database exists
    conn.query_drop(format!("CREATE DATABASE IF NOT EXISTS `{}`", db))?;
    conn.query_drop(format!("USE `{}`", db))?;

    // 2. Restore tables
    let mut stats = RestoreStats::default();
    for file in files {
        let sql = fs::read_to_string(&file)?;
        match conn.query_drop(sql) {
            Ok(_) => stats.restored += 1,
            Err(err) => {
                stats.failed += 1;
                eprintln!("❌ Failed to restore {}: {}", file.display(), err);
            }
        }
    }

    Ok(stats)
}

pub fn extract_zip(src: &Path, dst: &Path) -> Result<()> {
    if !src.exists() {
        bail!("Zip file does not exist: {}", src.display());
    }
    println!("📦 Detected zip file: {} — extracting to {}", src.display(), dst.display());

    let file = File::open(src)?;
    let mut archive = ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = dst.join(file.name());

        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(parent) = outpath.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut outfile = File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }

    println!("✅ Extraction completed: {}", dst.display());

    Ok(())
}
