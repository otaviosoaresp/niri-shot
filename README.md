# niri-shot

A screenshot tool for the [Niri](https://github.com/YaLTeR/niri) Wayland compositor with built-in annotation support.

## Features

- **Multiple capture modes**: Fullscreen, region selection, or window capture
- **Annotation tools**: Rectangle, circle, line, arrow, freehand drawing, text, blur, and highlight
- **Auto-copy**: Screenshots are automatically copied to clipboard
- **Zoom & Pan**: Navigate large screenshots with zoom (Ctrl+Scroll) and pan (Middle mouse / Shift+Right click)
- **Undo/Redo**: Full history support for annotations
- **GTK4 interface**: Modern, native Wayland experience

## Dependencies

- [grim](https://sr.ht/~emersion/grim/) - Screenshot utility for Wayland
- [slurp](https://github.com/emersion/slurp) - Region selection tool
- [wl-clipboard](https://github.com/bugaevc/wl-clipboard) - Clipboard utilities for Wayland
- GTK4
- A Nerd Font (optional, for toolbar icons)

### Arch Linux

```bash
sudo pacman -S grim slurp wl-clipboard gtk4
```

### Fedora

```bash
sudo dnf install grim slurp wl-clipboard gtk4
```

## Installation

### From source

```bash
git clone https://github.com/otaviosoaresp/niri-shot.git
cd niri-shot
cargo install --path .
```

### Cargo

```bash
cargo install niri-shot
```

## Usage

```bash
# Open the editor without capturing
niri-shot

# Capture fullscreen
niri-shot --fullscreen
niri-shot -f

# Capture a region (interactive selection)
niri-shot --region
niri-shot -r

# Capture a window/output
niri-shot --window
niri-shot -w
```

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+Z` | Undo |
| `Ctrl+Y` | Redo |
| `Ctrl+S` | Save screenshot |
| `Ctrl+C` | Copy to clipboard |
| `Ctrl++` | Zoom in |
| `Ctrl+-` | Zoom out |
| `Ctrl+0` | Reset zoom |
| `Ctrl+Scroll` | Zoom in/out |
| `Middle Mouse` | Pan |
| `Shift+Right Click` | Pan |

## Niri Configuration

Add the following to your niri config (`~/.config/niri/config.kdl`):

### Keybindings

```kdl
binds {
    Print { spawn "niri-shot" "--region"; }
    Ctrl+Print { spawn "niri-shot" "--fullscreen"; }
    Alt+Print { spawn "niri-shot" "--window"; }
}
```

### Floating Window Rule

To make niri-shot open as a floating window:

```kdl
window-rule {
    match app-id="com.github.niri-shot"
    open-floating true
}
```

## Tools

| Tool | Description |
|------|-------------|
| Select | Select and move/resize annotations |
| Rectangle | Draw rectangles |
| Circle | Draw circles/ellipses |
| Line | Draw straight lines |
| Arrow | Draw arrows |
| Freehand | Free drawing |
| Text | Add text annotations |
| Blur | Blur sensitive areas |
| Highlight | Highlight important areas |

## File Locations

- Screenshots: `~/Pictures/Screenshots/`
- Config: `~/.config/niri-shot/config.json`

## License

MIT License - see [LICENSE](LICENSE) for details.

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.
