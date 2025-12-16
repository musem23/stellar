# Stellar

```
  \|/
 --*--  Stellar
  /|\
```

A fast CLI tool to organize your files automatically.

## Features

- **3 Organization modes** - By category, date, or hybrid (category/year)
- **Watch mode** - Auto-organize new files as they appear
- **Smart renaming** - Clean filenames with accent support (élève → eleve)
- **Duplicate detection** - Find and remove duplicate files (SHA-256)
- **Undo support** - Revert the last operation
- **Recursive scan** - Organize subdirectories too
- **Dry-run preview** - See changes before applying
- **Progress bar & stats** - Visual feedback during operations
- **Safe** - Protects system folders, dev directories, and project folders

## Installation

### From source (recommended for beta)

```bash
# Clone
git clone https://github.com/musem23/stellar.git
cd stellar

# Build
cargo build --release

# Run
./target/release/stellar

# Or install globally
cargo install --path .
```

### macOS (after release)

```bash
curl -L https://github.com/musem23/stellar/releases/latest/download/stellar-macos.tar.gz | tar -xz
sudo mv stellar /usr/local/bin/
```

## Usage

### Interactive mode

```bash
stellar
```

```
? What would you like to do?
> [>] Organize a folder
  [~] Watch mode (auto-organize)
  [=] Find duplicates
  [<] Undo last operation
  [H] History
  [*] Settings
  [x] Exit
```

### CLI mode

```bash
# Organize Downloads by category
stellar ~/Downloads

# Organize by date
stellar ~/Downloads -m date

# Hybrid mode (Documents/2024, Images/2024...)
stellar ~/Downloads -m hybrid

# Recursive scan
stellar ~/Downloads -R

# Dry-run (preview only)
stellar ~/Downloads --dry-run

# Watch mode
stellar ~/Downloads --watch

# Custom rename mode
stellar ~/Downloads -r date-prefix
stellar ~/Downloads -r skip
```

### CLI Options

| Option | Short | Description |
|--------|-------|-------------|
| `--mode` | `-m` | Organization: `category`, `date`, `hybrid` |
| `--rename` | `-r` | Rename: `clean`, `date-prefix`, `skip` |
| `--recursive` | `-R` | Scan subdirectories |
| `--dry-run` | `-d` | Preview without changes |
| `--watch` | `-w` | Auto-organize new files |
| `--help` | `-h` | Show help |
| `--version` | `-V` | Show version |

## Organization Modes

### Category (default)
```
Downloads/
├── Documents/
├── Images/
├── Videos/
├── Audio/
└── Others/
```

### Date
```
Downloads/
├── 2024/
│   ├── 01-january/
│   ├── 02-february/
│   └── ...
└── 2025/
    └── ...
```

### Hybrid
```
Downloads/
├── Documents/
│   ├── 2024/
│   └── 2025/
├── Images/
│   ├── 2024/
│   └── 2025/
└── ...
```

## Rename Modes

| Mode | Example |
|------|---------|
| **Clean** | `Rapport FINAL (1).pdf` → `rapport-final.pdf` |
| **Clean** | `élève café.pdf` → `eleve-cafe.pdf` |
| **Date prefix** | `report.pdf` → `2024-01-15-report.pdf` |
| **Skip** | No renaming |

## Default Categories

| Category | Extensions |
|----------|------------|
| Documents | pdf, doc, docx, txt, md, odt, rtf, xlsx, xls, csv, pptx, ppt |
| Images | png, jpg, jpeg, gif, webp, svg, bmp, ico, tiff, heic, psd |
| Videos | mp4, mkv, avi, mov, wmv, flv, webm, m4v |
| Audio | mp3, wav, flac, aac, ogg, wma, m4a |
| Archives | zip, tar, gz, rar, 7z, bz2, xz |
| Code | rs, js, ts, py, html, css, json, yaml, toml, sh |
| Executables | exe, msi, app, dmg, pkg, deb, rpm, iso |
| Fonts | ttf, otf, woff, woff2, eot |
| Ebooks | epub, mobi, azw, azw3, fb2, djvu |

Categories are customizable in Settings or via `~/.config/stellar/stellar.toml`.

## Protected Folders

Stellar refuses to organize:
- System folders (`/`, `/System`, `/Library`, etc.)
- User sensitive folders (`.ssh`, `.gnupg`, `.config`)
- Project folders (containing `.git`, `package.json`, `Cargo.toml`, etc.)
- Dev folders (`node_modules`, `target`, `venv`, etc.)

## Configuration

Config file: `~/.config/stellar/stellar.toml`

```toml
[preferences]
organization_mode = 0  # 0=category, 1=date, 2=hybrid
rename_mode = 0        # 0=clean, 1=date-prefix, 2=skip

[categories]
Documents = ["pdf", "doc", "docx", "txt"]
Images = ["png", "jpg", "jpeg", "gif"]
# ... add your own
```

## License

MIT - [@musem23](https://github.com/musem23)
