# Molniya

**Molniya** is a Rust-based CLI tool for restoring MySQL database backups with support for:

- Zipped or folder contains `.sql.gz` backups
- Table exclusion presets
- Dry-run mode for safe preview
- Safety confirmation via `--yes`
- Preset management (`add`, `edit`, `list`, `remove`)

It is designed for **database administrators, developers, and automation scripts**.

---
## Dependencies

*   Rust >= 1.70
*   MySQL server or compatible (MariaDB, etc)
*   .sql.gz backups

## Installation

### Option 1: Install from Prebuilt Binary
Download the binary from [release](https://github.com/sqyo17/molniya/releases)
```bash
chmod +x molniya
sudo install -m 755 molniya /usr/local/bin/molniya
```

### Option 2: Build from Source

```bash
git clone https://github.com/sqyo17/molniya.git
cd molniya
cargo build --release
sudo install -m 755 target/release/molniya /usr/local/bin/molniya
```
Verify
```
molniya --version
```

## Configuration

Molniya uses environment variables to connect to MySQL:

| Variable       | Default   | Description                 |
|----------------|-----------|-----------------------------|
| `MYSQL_USER`   | root      | MySQL username (required)   |
| `MYSQL_PASSWORD` | null      | MySQL password             |
| `MYSQL_HOST`   | 127.0.0.1 | MySQL host               |
| `MYSQL_PORT`   | 3306      | MySQL port                  |

You can set them in a `.env` file in the project folder:

```env
MYSQL_USER=root
MYSQL_PASSWORD=
MYSQL_HOST=127.0.0.1
MYSQL_PORT=3306
```

## Usage

Molniya has two main commands: `restore` and `preset`.

```
molniya <COMMAND> [OPTIONS]
```

### Restore

Restore a MySQL database from a backup folder:
```
molniya restore <BACKUP_FOLDER or ZIP_FILE> --db <DB_NAME> 
[--preset <PRESET>] [--dry-run] [--yes]
``` 

Option

| Option | Description                                     |
| ----- | ----------------------------------------------- |
| db    | Name of the database to restore                 |
| preset | Use a preset to exclude tables                  |
| dry-run | Show what would be restored without changing DB |
| yes   | Skip confirmation prompt                        |

### Presets

Presets allow you to define tables to exclude during restore:

```
molniya preset add <NAME>
molniya preset edit <NAME>
molniya preset list
molniya preset remove <NAME>
``` 

*   `add`: interactively create a new preset
*   `edit`: edit an existing preset
*   `list`: show all presets
*   `remove`: delete a preset

Flags
-----

*   `--dry-run` – preview restore without executing SQL
*   `--yes` – confirm restore automatically (non-interactive)
*   `-h, --help` – show help

Examples
--------

### 1\. Basic Restore
```
molniya restore path/to/folder/of/sql.qz --db mydb
```

### 2\. Restore With Preset
```
molniya restore path/to/folder/of/sql.qz --db mydb --preset mypreset
```

### 3\. Dry Run
```
molniya restore path/to/folder/of/sql.qz --db mydb --preset mypreset --dry-run
```

### 4\. Non-Interactive Restore
```
molniya restore path/to/folder/of/sql.qz --db mydb --preset mypreset --yes
```

### 5\. Preset Management
```
molniya preset add mypreset
molniya preset list
molniya preset edit mypreset
molniya preset remove mypreset
```
