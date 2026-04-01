# Light

A super lightweight, resource-efficient web browser built with Rust.

Firefox and Chrome are great, but they consume gigabytes of RAM for a single tab. Light strips away everything you don't need — no extensions, no devtools, no sync, no AI features — and gives you a fast, minimal browser that just works.

## Features

- Vertical tab sidebar
- Address bar with navigation (back/forward/reload)
- Bookmarks with persistent storage
- Configurable home page
- Keyboard shortcuts (Cmd/Ctrl + T, W, L, R)
- Page title tracking
- Native webview rendering (WebKit on macOS, WebView2 on Windows, WebKitGTK on Linux)

## Memory Usage

Tested with YouTube open:

| Browser | RAM |
|---------|-----|
| Firefox | ~2.5 GB |
| **Light** | **~1.2 GB** |

Light uses ~52% less memory. The browser overhead (everything except the page's own JS/DOM) is ~250 MB vs Firefox's ~1.6 GB.

## Build

```bash
cargo build --release
```

### Linux Dependencies

```bash
sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev
```

## Run

```bash
cargo run --release
```

## Configuration

Settings and bookmarks are stored in your system config directory:

- **macOS**: `~/Library/Application Support/light/`
- **Linux**: `~/.config/light/`
- **Windows**: `%APPDATA%\light\`

Files:
- `settings.json` — home page URL
- `bookmarks.json` — saved bookmarks

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| Cmd/Ctrl + T | New tab |
| Cmd/Ctrl + W | Close tab |
| Cmd/Ctrl + L | Focus address bar |
| Cmd/Ctrl + R | Reload |

## Tech Stack

- **Rust** — memory-safe, zero GC overhead
- **tao** — cross-platform windowing
- **wry** — native webview wrapper
- **Engine trait** — abstracted for future engine swapability

## License

MIT
