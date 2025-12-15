# Stellar

```
  \|/
 --*--  Stellar
  /|\
```

A fast CLI tool to organize your files automatically.

## Features

- **Organize by category** - Sort files into Documents, Images, Videos, Audio, etc.
- **Organize by date** - Sort files into year/month folders (2024/01-january)
- **Watch mode** - Auto-organize new files as they appear
- **Smart renaming** - Clean filenames (lowercase, dashes) or add date prefix
- **Customizable** - Add/edit categories and save preferences
- **Safe** - Protects system folders and dev directories

## Installation

### macOS

```bash
# Download and extract
curl -L https://github.com/musem23/stellar/releases/latest/download/stellar-macos.tar.gz | tar -xz

# Run
./stellar-macos

# Or install globally
sudo mv stellar-macos /usr/local/bin/stellar
stellar
```

If blocked by Gatekeeper:
```bash
xattr -d com.apple.quarantine stellar-macos
```

### Windows

Download `stellar-windows.exe` from [releases](https://github.com/musem23/stellar/releases) and run it.

## Usage

```
? What would you like to do?
> [>] Organize a folder
  [~] Watch mode (auto-organize)
  [*] Settings
  [x] Exit
```

### Organize a folder

1. Select a folder from your home directory
2. Choose to use saved preferences or customize
3. Select organization mode (by category or by date)
4. Select rename mode (clean, date prefix, or skip)
5. Preview and confirm

### Watch mode

Monitors a folder and automatically organizes new files as they appear.

### Settings

- View/add/edit/remove categories
- Set default organization mode
- Set default rename mode
- Save preferences to `~/.config/stellar/stellar.toml`

## Default Categories

| Category  | Extensions                                    |
|-----------|-----------------------------------------------|
| Documents | pdf, doc, docx, txt, odt, rtf, xls, xlsx, ppt, pptx, csv |
| Images    | png, jpg, jpeg, gif, webp, svg, bmp, ico, tiff, heic |
| Videos    | mp4, mkv, avi, mov, wmv, flv, webm, m4v       |
| Audio     | mp3, wav, flac, aac, ogg, wma, m4a            |
| Archives  | zip, tar, gz, rar, 7z, bz2, xz                |
| Code      | rs, js, ts, py, html, css, json, yaml, toml, md, sh |
| Fonts     | ttf, otf, woff, woff2, eot                    |
| Ebooks    | epub, mobi, azw, azw3, fb2, djvu              |

## Build from source

```bash
# Clone
git clone https://github.com/musem23/stellar.git
cd stellar

# Build
cargo build --release

# Run
./target/release/stellar
```

## License

MIT
