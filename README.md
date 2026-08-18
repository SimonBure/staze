
# staze

A terminal time tracker. Start sessions, label them, and review your time as a bar chart — all from the keyboard. 

Cute bonus: you'll adpot **Staz**, a small creature that lives as your work.

<!-- Demo gif — render with `vhs demo.tape` (https://github.com/charmbracelet/vhs) -->
![staze demo](demo.gif)

<p align="center">
    <img src="https://img.shields.io/crates/v/staze.svg?style=for-the-badge" alt="Crates.io">
    <a href="https://github.com/SimonBure/staze"><img src="https://img.shields.io/github/stars/SimonBure/staze?style=for-the-badge&label=Stars%20&logo=github&logoColor=white" alt="Staze stars" /></a>
</p>

## Install

### From source (any platform)

```sh
cargo install staze
```

### macOS / Linux (prebuilt binary)

```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/SimonBure/staze/releases/latest/download/staze-installer.sh | sh
```

### Windows (PowerShell)

```powershell
powershell -ExecutionPolicy ByPass -c "irm https://github.com/SimonBure/staze/releases/latest/download/staze-installer.ps1 | iex"
```

Requires no system dependencies (SQLite is bundled).

## Usage

```sh
staze
```

| Key | Action |
|-----|--------|
| `←` `→` / `h` `l` | Navigate menu |
| `↑` `↓` / `k` `j` | Navigate list |
| `Enter` | Select |
| `Q` | Quit / back |

Vim-style `h` `j` `k` `l` work as aliases for the arrow keys everywhere.

Sessions are stored in `~/.local/share/staze/staze.db`.

## Configuration

Optional config at `~/.config/staze/config.toml`:

```toml
db_path = "/custom/path/to/staze.db"
```

## License

MIT
