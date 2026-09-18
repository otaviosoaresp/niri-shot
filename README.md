# niri-shot

[![CI](https://github.com/otaviosoaresp/niri-shot/actions/workflows/ci.yml/badge.svg)](https://github.com/otaviosoaresp/niri-shot/actions/workflows/ci.yml)

A screenshot tool for the [niri](https://github.com/niri-wm/niri) Wayland compositor with a built-in annotation editor.

## Features

- **niri-native capture**: pick a window with the mouse, or capture the focused monitor, through niri IPC. niri renders only the picked window, so windows covering it do not appear.
- **Region capture**: interactive selection with `slurp`; the last region is highlighted on the next run
- **Annotation tools**: rectangle, circle, line, arrow, freehand drawing, text, blur, and highlight
- **Clipboard**: command-line captures are copied right away; Ctrl+C copies the annotated image
- **Zoom and pan**: Ctrl+Scroll to zoom; middle mouse drag or Shift+Right drag to pan
- **Undo/redo**: drawing, moving, resizing, rotating and deleting annotations are all undoable
- **GTK4 interface**

## Requirements

- niri 25.11 or newer (window and monitor capture use `niri msg`)
- GTK 4
- [wl-clipboard](https://github.com/bugaevc/wl-clipboard)
- [grim](https://sr.ht/~emersion/grim/) and [slurp](https://github.com/emersion/slurp), for region capture only
- A Nerd Font (optional, for toolbar icons)

### Arch Linux

```bash
sudo pacman -S gtk4 wl-clipboard grim slurp
```

### Fedora

```bash
sudo dnf install gtk4 wl-clipboard grim slurp
```

## Installation

### Prebuilt binary

Each [release](https://github.com/otaviosoaresp/niri-shot/releases) ships `niri-shot-x86_64-linux.tar.gz` with a `.sha256` checksum. The binary is built on Ubuntu 24.04 and needs glibc 2.39 or newer; GTK 4 releases older than 4.14 are untested.

```bash
curl -LO https://github.com/otaviosoaresp/niri-shot/releases/latest/download/niri-shot-x86_64-linux.tar.gz
curl -LO https://github.com/otaviosoaresp/niri-shot/releases/latest/download/niri-shot-x86_64-linux.tar.gz.sha256
sha256sum -c niri-shot-x86_64-linux.tar.gz.sha256
tar -xzf niri-shot-x86_64-linux.tar.gz
install -Dm755 niri-shot ~/.local/bin/niri-shot
```

### From source

```bash
git clone https://github.com/otaviosoaresp/niri-shot.git
cd niri-shot
cargo install --path .
```

### Nix flake

The Nix package puts `grim`, `slurp` and `wl-copy` on its own `PATH`.

As a flake input (e.g. in NixOS / Home Manager):

```nix
{
  inputs = {
    niri-shot.url = "github:otaviosoaresp/niri-shot";
  };

  outputs = { self, nixpkgs, niri-shot, ... }: {
    # NixOS module
    environment.systemPackages = [ niri-shot.packages.x86_64-linux.default ];

    # Home Manager
    home.packages = [ niri-shot.packages.x86_64-linux.default ];
  };
}
```

Or run directly without installing:

```bash
nix run github:otaviosoaresp/niri-shot
```

### Nix development shell

```bash
git clone https://github.com/otaviosoaresp/niri-shot.git
cd niri-shot
nix develop
cargo build --release
```

Or with direnv:

```bash
direnv allow
cargo build --release
```

## Usage

```bash
# Open the editor without capturing
niri-shot

# Capture the focused monitor
niri-shot --fullscreen
niri-shot -f

# Capture a region (interactive selection)
niri-shot --region
niri-shot -r

# Pick a window with the mouse (Esc cancels)
niri-shot --window
niri-shot -w
```

Cancelling a selection exits without opening the editor. If a capture fails, the editor opens and shows the error in its status bar.

Window and monitor captures are taken by niri itself, so niri also copies the image to the clipboard and may show its own screenshot notification.

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
| `Delete` / `Backspace` | Delete the selected annotation |
| `Middle Mouse` | Pan |
| `Shift+Right Click` | Pan |

## niri Configuration

Add the following to your niri config (`~/.config/niri/config.kdl`).

### Keybindings

niri's default config binds these keys to its own screenshot actions; replace those lines.

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
| Select | Select and move/resize/rotate annotations |
| Rectangle | Draw rectangles |
| Circle | Draw circles/ellipses |
| Line | Draw straight lines |
| Arrow | Draw arrows |
| Freehand | Free drawing |
| Text | Add text annotations |
| Blur | Cover sensitive areas with an opaque pattern |
| Highlight | Highlight important areas |

## File Locations

- Screenshots: `~/Pictures/Screenshots/` (your XDG pictures directory)
- Last region: `~/.cache/niri-shot/last-region`

## License

MIT License - see [LICENSE](LICENSE) for details.

## Contributing

Issues and pull requests are welcome. Pull requests are squash-merged and the PR title becomes the changelog entry, so PR titles must follow [Conventional Commits](https://www.conventionalcommits.org/) (`feat: ...`, `fix: ...`, `docs: ...`). CI runs `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`, `nix build` and a PR title check.
