[![Crates.io](https://img.shields.io/crates/v/staze.svg)](https://crates.io/crates/staze)
# staze

A terminal time tracker. Start sessions, label them, and review your time as a bar chart — all from the keyboard. Adpot Staz, a cute creature that live with your work.

<!-- Demo gif — render with `vhs demo.tape` (https://github.com/charmbracelet/vhs) -->
![staze demo](demo.gif)

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
| `←` `→` | Navigate menu |
| `Enter` | Select |
| `Q` | Quit / back |

Sessions are stored in `~/.local/share/staze/staze.db`.

## Configuration

Optional config at `~/.config/staze/config.toml`:

```toml
db_path = "/custom/path/to/staze.db"
```

## License

MIT
