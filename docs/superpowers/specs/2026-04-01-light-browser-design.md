# Light Browser — Design Spec

## Overview

Light is a super lightweight, resource-efficient web browser. It renders web pages using the OS's native webview engine, wrapped in a minimal native UI. The goal is to eliminate the hundreds of megabytes of overhead that Firefox/Chrome impose through multi-process architectures, extensions, devtools, sync, and other features most users don't need.

## Goals

- **Minimal memory footprint** — target 80-85% less RAM than Firefox for the same page
- **Cross-platform** — macOS, Linux, Windows
- **Just a browser** — no bookmarks, history, extensions, devtools, settings page, search engine integration, or any feature not listed below
- **Future engine swapability** — abstract the rendering engine behind a trait so we can swap from wry to CEF or others later

## Tech Stack

- **Language:** Rust
- **Windowing/UI:** `iced` (cross-platform native GUI)
- **Web engine:** `wry` (native webview wrapper — WebKit on macOS, WebView2 on Windows, WebKitGTK on Linux)
- **Windowing backend for wry:** `tao` (comes with wry)

## Architecture

```
┌─────────────────────────────────────┐
│           Light Browser             │
├─────────────────────────────────────┤
│  main.rs                            │
│  ├── App (entry point, event loop)  │
│  │                                  │
│  ├── UI Layer (iced)                │
│  │   ├── TabBar                     │
│  │   ├── AddressBar                 │
│  │   └── NavigationButtons          │
│  │                                  │
│  ├── Engine Trait                   │
│  │   └── WryEngine (impl)          │
│  │                                  │
│  └── TabManager                     │
│       └── tracks tab state          │
└─────────────────────────────────────┘
```

### Data Flow

1. User interacts with native UI (clicks tab, types URL, presses button)
2. UI layer emits a message (e.g., `Navigate("https://...")`)
3. App dispatches message to the appropriate engine instance via `TabManager`
4. Engine loads/renders the content in the corresponding webview
5. Engine reports state changes back (page title, loading status) → UI updates

## Module Structure

```
src/
├── main.rs              # Entry point, wires everything together
├── app.rs               # App state, message handling, iced Application impl
├── ui/
│   ├── mod.rs
│   ├── tab_bar.rs       # Tab bar widget with drag-reorder
│   ├── address_bar.rs   # URL input field
│   └── nav_buttons.rs   # Back, forward, reload buttons
├── engine/
│   ├── mod.rs           # WebEngine trait definition
│   └── wry.rs           # Wry-based implementation
├── tab.rs               # Tab state (id, title, url, loading status)
└── keys.rs              # Keyboard shortcut handling
```

## Engine Trait

```rust
pub trait WebEngine {
    fn create_webview(&mut self, tab_id: TabId, url: &str) -> Result<()>;
    fn close_webview(&mut self, tab_id: TabId) -> Result<()>;
    fn navigate(&mut self, tab_id: TabId, url: &str) -> Result<()>;
    fn go_back(&mut self, tab_id: TabId) -> Result<()>;
    fn go_forward(&mut self, tab_id: TabId) -> Result<()>;
    fn reload(&mut self, tab_id: TabId) -> Result<()>;
    fn set_visible(&mut self, tab_id: TabId, visible: bool) -> Result<()>;
}
```

Each tab gets its own webview instance. `set_visible` handles tab switching — only the active tab's webview is shown.

## Tab Management

```rust
pub struct Tab {
    pub id: TabId,
    pub title: String,       // from page <title>, fallback to URL
    pub url: String,         // current URL
    pub is_loading: bool,
}

pub struct TabManager {
    tabs: Vec<Tab>,
    active: usize,           // index of active tab
    next_id: u64,            // monotonic counter for TabId
}
```

### Tab Behavior

- New tab opens `about:blank` and focuses the address bar
- Closing the last tab quits the app
- Drag-reorder changes the order in `tabs: Vec<Tab>` — no other side effects
- Tab title updates when the page emits a title change event from the webview

## UI Layout

```
┌──────────────────────────────────────────────┐
│ [Tab 1] [Tab 2] [Tab 3]  [+]                │  ← tab bar (~30px)
├──────────────────────────────────────────────┤
│ [←] [→] [↻]  [ https://example.com       ]  │  ← nav bar (~36px)
├──────────────────────────────────────────────┤
│                                              │
│              Web content                     │  ← webview fills rest
│                                              │
└──────────────────────────────────────────────┘
```

- **Tab bar:** horizontal row of tab buttons, active tab highlighted, `[+]` button at end. Drag-reorder via mouse.
- **Nav bar:** back/forward/reload as icon buttons, address bar takes remaining width. Pressing Enter navigates. No search engine integration — input is treated as a URL.
- **Webview:** fills all remaining vertical space.

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| Cmd/Ctrl+T | New tab |
| Cmd/Ctrl+W | Close current tab |
| Cmd/Ctrl+L | Focus address bar |
| Cmd/Ctrl+R | Reload current tab |

## Non-Goals (explicitly excluded)

- Bookmarks
- History
- Extensions / plugins
- Developer tools
- Settings page
- Search engine integration
- Password manager
- Sync
- PDF viewer
- Reader mode
- AI features
- Telemetry
