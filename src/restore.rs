use crate::config::load_config;
use crate::db::{mysql_pool, test_connection};
use anyhow::Result;
use flate2::read::GzDecoder;
use mysql::prelude::Queryable;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

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

    // 0. Test connection
    test_connection()?;

    // 1. Resolve exclude tables from preset
    let exclude_tables = if let Some(name) = preset {
        let cfg = load_config()?;
        let preset = cfg.presets
            .get(&name)
            .ok_or_else(|| anyhow::anyhow!("Preset '{}' not found", name))?;

        preset.exclude_tables.clone()
    } else {
        Vec::new()
    };

    // 2. Extract backup → Vec<PathBuf>
    let (sql_files, skipped) = extract_backup(&backup, &exclude_tables)?;

    // 2,5. Dry run
    // Summary
    println!("Database        : {}", db);
    println!("Backup folder   : {}", backup.display());
    println!("Tables to restore: {}", sql_files.len());
    println!("Tables skipped  : {}", skipped);

    if dry_run {
        println!("Dry run enabled — no changes will be made");
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
            println!("Aborted by user");
            return Ok(());
        }
    }
    restore_sql(&db, sql_files)?;
    
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

fn restore_sql(db: &str, files: Vec<PathBuf>) -> Result<RestoreStats> {
    let pool = mysql_pool()?;
    let mut conn = pool.get_conn()?;

    conn.query_drop(format!("CREATE DATABASE IF NOT EXISTS `{}`", db))?;
    conn.query_drop(format!("USE `{}`", db))?;

    let mut stats = RestoreStats::default();

    for file in files {
        let sql = fs::read_to_string(&file)?;

        match conn.query_drop(sql) {
            Ok(_) => stats.restored += 1,
            Err(err) => {
                stats.failed += 1;
                eprintln!(
                    "Failed to restore {}: {}",
                    file.display(),
                    err
                );
            }
        }
    }

    Ok(stats)
}
