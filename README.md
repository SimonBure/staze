# staze

A terminal time tracker. Start sessions, label them, and review your time as a bar chart — all from the keyboard.

## Install

```sh
cargo install staze
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
