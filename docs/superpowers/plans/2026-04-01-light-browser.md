# Light Browser Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a minimal, resource-efficient cross-platform web browser using Rust with native webviews.

**Architecture:** Single-process app using `winit` for windowing and `wry` for webviews. The browser chrome (tab bar + nav bar) is a lightweight HTML/CSS webview at the top of the window, communicating with Rust via IPC. Each tab is a separate wry webview positioned below the chrome. An engine trait abstracts all webview operations for future swapability.

**Tech Stack:** Rust, wry (webview), winit (windowing), serde/serde_json (IPC serialization)

**Design spec:** `docs/superpowers/specs/2026-04-01-light-browser-design.md`

**Note on iced:** The spec mentions `iced` for native UI, but iced owns its own event loop which conflicts with wry's winit backend. Using an HTML chrome webview is the pragmatic solution — it's ~2-3MB overhead, fully cross-platform, and keeps the codebase simple. The engine trait boundary is preserved exactly as designed.

---

## File Structure

```
Cargo.toml                  # Dependencies: wry, winit, serde, serde_json
src/
├── main.rs                 # Entry point, creates App and runs event loop
├── app.rs                  # App struct, ApplicationHandler impl, event dispatch
├── tab.rs                  # TabId, Tab, TabManager — all tab state
├── engine/
│   ├── mod.rs              # WebEngine trait definition + re-exports
│   └── wry_engine.rs       # WryEngine: creates/manages wry webviews
├── ipc.rs                  # IPC message types (Chrome→Rust, Rust→Chrome)
├── chrome.rs               # Generates the chrome HTML/CSS/JS string
├── url.rs                  # URL normalization (user input → valid URL)
└── keys.rs                 # Keyboard shortcut detection from winit events
```

---

### Task 1: Project scaffolding

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`

- [ ] **Step 1: Create Cargo.toml**

```toml
[package]
name = "light"
version = "0.1.0"
edition = "2024"

[dependencies]
wry = "0.50"
winit = "0.30"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

- [ ] **Step 2: Create minimal main.rs that opens a window**

```rust
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

#[derive(Default)]
struct App {
    window: Option<Window>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attrs = Window::default_attributes().with_title("Light");
        let window = event_loop.create_window(attrs).unwrap();
        self.window = Some(window);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        if let WindowEvent::CloseRequested = event {
            event_loop.exit();
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let mut app = App::default();
    event_loop.run_app(&mut app).unwrap();
}
```

- [ ] **Step 3: Build and verify window opens**

Run: `cargo run`
Expected: A blank window titled "Light" appears. Close it to exit.

- [ ] **Step 4: Commit**

```bash
git add Cargo.toml src/main.rs
git commit -m "feat: project scaffolding with blank window"
```

---

### Task 2: URL normalization

**Files:**
- Create: `src/url.rs`
- Modify: `src/main.rs` (add `mod url;`)

- [ ] **Step 1: Write tests for URL normalization**

```rust
// src/url.rs

pub fn normalize_url(input: &str) -> String {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_url_unchanged() {
        assert_eq!(normalize_url("https://example.com"), "https://example.com");
    }

    #[test]
    fn http_url_unchanged() {
        assert_eq!(normalize_url("http://example.com"), "http://example.com");
    }

    #[test]
    fn adds_https_to_domain() {
        assert_eq!(normalize_url("example.com"), "https://example.com");
    }

    #[test]
    fn adds_https_to_domain_with_path() {
        assert_eq!(
            normalize_url("example.com/page"),
            "https://example.com/page"
        );
    }

    #[test]
    fn trims_whitespace() {
        assert_eq!(normalize_url("  https://example.com  "), "https://example.com");
    }

    #[test]
    fn localhost_with_port() {
        assert_eq!(normalize_url("localhost:3000"), "http://localhost:3000");
    }

    #[test]
    fn about_blank() {
        assert_eq!(normalize_url("about:blank"), "about:blank");
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --lib url`
Expected: All 7 tests FAIL with "not yet implemented"

- [ ] **Step 3: Implement normalize_url**

Replace the `todo!()` in `src/url.rs` with:

```rust
pub fn normalize_url(input: &str) -> String {
    let trimmed = input.trim();

    if trimmed.starts_with("about:") {
        return trimmed.to_string();
    }

    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        return trimmed.to_string();
    }

    if trimmed.starts_with("localhost") || trimmed.starts_with("127.0.0.1") {
        return format!("http://{trimmed}");
    }

    format!("https://{trimmed}")
}
```

- [ ] **Step 4: Add module declaration to main.rs**

Add at the top of `src/main.rs`:
```rust
mod url;
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test --lib url`
Expected: All 7 tests PASS

- [ ] **Step 6: Commit**

```bash
git add src/url.rs src/main.rs
git commit -m "feat: add URL normalization"
```

---

### Task 3: Tab state and TabManager

**Files:**
- Create: `src/tab.rs`
- Modify: `src/main.rs` (add `mod tab;`)

- [ ] **Step 1: Write Tab and TabId types, then TabManager tests**

```rust
// src/tab.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TabId(pub u64);

#[derive(Debug, Clone)]
pub struct Tab {
    pub id: TabId,
    pub title: String,
    pub url: String,
    pub is_loading: bool,
}

pub struct TabManager {
    tabs: Vec<Tab>,
    active: usize,
    next_id: u64,
}

impl TabManager {
    pub fn new() -> Self {
        Self {
            tabs: Vec::new(),
            active: 0,
            next_id: 0,
        }
    }

    pub fn create_tab(&mut self, url: &str) -> TabId {
        todo!()
    }

    pub fn close_tab(&mut self, id: TabId) -> Option<TabId> {
        // Returns the new active tab id, or None if no tabs remain
        todo!()
    }

    pub fn active_tab(&self) -> Option<&Tab> {
        todo!()
    }

    pub fn set_active(&mut self, id: TabId) {
        todo!()
    }

    pub fn reorder(&mut self, from: usize, to: usize) {
        todo!()
    }

    pub fn update_title(&mut self, id: TabId, title: String) {
        todo!()
    }

    pub fn update_url(&mut self, id: TabId, url: String) {
        todo!()
    }

    pub fn set_loading(&mut self, id: TabId, loading: bool) {
        todo!()
    }

    pub fn tabs(&self) -> &[Tab] {
        &self.tabs
    }

    pub fn active_id(&self) -> Option<TabId> {
        self.tabs.get(self.active).map(|t| t.id)
    }

    pub fn is_empty(&self) -> bool {
        self.tabs.is_empty()
    }

    fn find_index(&self, id: TabId) -> Option<usize> {
        self.tabs.iter().position(|t| t.id == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_tab_returns_unique_ids() {
        let mut mgr = TabManager::new();
        let id1 = mgr.create_tab("https://a.com");
        let id2 = mgr.create_tab("https://b.com");
        assert_ne!(id1, id2);
    }

    #[test]
    fn create_tab_sets_active_to_new_tab() {
        let mut mgr = TabManager::new();
        mgr.create_tab("https://a.com");
        let id2 = mgr.create_tab("https://b.com");
        assert_eq!(mgr.active_id(), Some(id2));
    }

    #[test]
    fn active_tab_returns_correct_tab() {
        let mut mgr = TabManager::new();
        let id = mgr.create_tab("https://example.com");
        let tab = mgr.active_tab().unwrap();
        assert_eq!(tab.id, id);
        assert_eq!(tab.url, "https://example.com");
        assert_eq!(tab.title, "New Tab");
    }

    #[test]
    fn close_tab_returns_new_active() {
        let mut mgr = TabManager::new();
        let id1 = mgr.create_tab("https://a.com");
        let id2 = mgr.create_tab("https://b.com");
        let new_active = mgr.close_tab(id2);
        assert_eq!(new_active, Some(id1));
    }

    #[test]
    fn close_last_tab_returns_none() {
        let mut mgr = TabManager::new();
        let id = mgr.create_tab("https://a.com");
        let result = mgr.close_tab(id);
        assert_eq!(result, None);
        assert!(mgr.is_empty());
    }

    #[test]
    fn close_active_tab_activates_previous() {
        let mut mgr = TabManager::new();
        let id1 = mgr.create_tab("https://a.com");
        let _id2 = mgr.create_tab("https://b.com");
        let id3 = mgr.create_tab("https://c.com");
        mgr.set_active(id1);
        // Close the first tab — active should move to the tab now at index 0
        let new_active = mgr.close_tab(id1);
        assert!(new_active.is_some());
        assert!(mgr.active_tab().is_some());
    }

    #[test]
    fn set_active_changes_active_tab() {
        let mut mgr = TabManager::new();
        let id1 = mgr.create_tab("https://a.com");
        let _id2 = mgr.create_tab("https://b.com");
        mgr.set_active(id1);
        assert_eq!(mgr.active_id(), Some(id1));
    }

    #[test]
    fn reorder_moves_tab() {
        let mut mgr = TabManager::new();
        let id1 = mgr.create_tab("https://a.com");
        let _id2 = mgr.create_tab("https://b.com");
        let id3 = mgr.create_tab("https://c.com");
        mgr.reorder(2, 0);
        assert_eq!(mgr.tabs()[0].id, id3);
        assert_eq!(mgr.tabs()[1].id, id1);
    }

    #[test]
    fn update_title_changes_title() {
        let mut mgr = TabManager::new();
        let id = mgr.create_tab("https://example.com");
        mgr.update_title(id, "Example".to_string());
        assert_eq!(mgr.active_tab().unwrap().title, "Example");
    }

    #[test]
    fn update_url_changes_url() {
        let mut mgr = TabManager::new();
        let id = mgr.create_tab("https://example.com");
        mgr.update_url(id, "https://other.com".to_string());
        assert_eq!(mgr.active_tab().unwrap().url, "https://other.com");
    }

    #[test]
    fn set_loading_changes_state() {
        let mut mgr = TabManager::new();
        let id = mgr.create_tab("https://example.com");
        mgr.set_loading(id, true);
        assert!(mgr.active_tab().unwrap().is_loading);
        mgr.set_loading(id, false);
        assert!(!mgr.active_tab().unwrap().is_loading);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --lib tab`
Expected: All tests FAIL with "not yet implemented"

- [ ] **Step 3: Implement TabManager methods**

Replace all `todo!()` bodies in `src/tab.rs`:

```rust
    pub fn create_tab(&mut self, url: &str) -> TabId {
        let id = TabId(self.next_id);
        self.next_id += 1;
        self.tabs.push(Tab {
            id,
            title: "New Tab".to_string(),
            url: url.to_string(),
            is_loading: false,
        });
        self.active = self.tabs.len() - 1;
        id
    }

    pub fn close_tab(&mut self, id: TabId) -> Option<TabId> {
        let Some(idx) = self.find_index(id) else {
            return self.active_id();
        };
        self.tabs.remove(idx);
        if self.tabs.is_empty() {
            return None;
        }
        if self.active >= self.tabs.len() {
            self.active = self.tabs.len() - 1;
        } else if self.active > idx {
            self.active -= 1;
        }
        self.active_id()
    }

    pub fn active_tab(&self) -> Option<&Tab> {
        self.tabs.get(self.active)
    }

    pub fn set_active(&mut self, id: TabId) {
        if let Some(idx) = self.find_index(id) {
            self.active = idx;
        }
    }

    pub fn reorder(&mut self, from: usize, to: usize) {
        if from >= self.tabs.len() || to >= self.tabs.len() {
            return;
        }
        let active_id = self.active_id();
        let tab = self.tabs.remove(from);
        self.tabs.insert(to, tab);
        if let Some(id) = active_id {
            if let Some(idx) = self.find_index(id) {
                self.active = idx;
            }
        }
    }

    pub fn update_title(&mut self, id: TabId, title: String) {
        if let Some(idx) = self.find_index(id) {
            self.tabs[idx].title = title;
        }
    }

    pub fn update_url(&mut self, id: TabId, url: String) {
        if let Some(idx) = self.find_index(id) {
            self.tabs[idx].url = url;
        }
    }

    pub fn set_loading(&mut self, id: TabId, loading: bool) {
        if let Some(idx) = self.find_index(id) {
            self.tabs[idx].is_loading = loading;
        }
    }
```

- [ ] **Step 4: Add module declaration to main.rs**

Add to `src/main.rs`:
```rust
mod tab;
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test --lib tab`
Expected: All 11 tests PASS

- [ ] **Step 6: Commit**

```bash
git add src/tab.rs src/main.rs
git commit -m "feat: add Tab and TabManager with tests"
```

---

### Task 4: IPC message types

**Files:**
- Create: `src/ipc.rs`
- Modify: `src/main.rs` (add `mod ipc;`)

- [ ] **Step 1: Write IPC types and serialization tests**

```rust
// src/ipc.rs

use serde::{Deserialize, Serialize};

/// Messages sent from the chrome webview (JS) to Rust via window.ipc.postMessage
#[derive(Debug, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum ChromeToApp {
    Navigate { url: String },
    NewTab,
    CloseTab { id: u64 },
    SwitchTab { id: u64 },
    GoBack,
    GoForward,
    Reload,
    ReorderTab { from: usize, to: usize },
}

/// Messages sent from Rust to the chrome webview via evaluate_script
#[derive(Debug, Serialize, PartialEq)]
#[serde(tag = "type")]
pub enum AppToChrome {
    TabCreated { id: u64, title: String, url: String },
    TabClosed { id: u64 },
    TabUpdated { id: u64, title: String, url: String, is_loading: bool },
    ActiveTabChanged { id: u64 },
    AllTabs { tabs: Vec<TabInfo>, active_id: u64 },
}

#[derive(Debug, Serialize, PartialEq)]
pub struct TabInfo {
    pub id: u64,
    pub title: String,
    pub url: String,
    pub is_loading: bool,
}

impl AppToChrome {
    /// Returns a JS expression that calls handleMessage on the chrome webview
    pub fn to_js_call(&self) -> String {
        let json = serde_json::to_string(self).unwrap();
        format!("handleMessage({json})")
    }
}

pub fn parse_chrome_message(body: &str) -> Result<ChromeToApp, serde_json::Error> {
    serde_json::from_str(body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_navigate() {
        let msg = r#"{"type":"Navigate","url":"https://example.com"}"#;
        let parsed = parse_chrome_message(msg).unwrap();
        assert_eq!(parsed, ChromeToApp::Navigate { url: "https://example.com".to_string() });
    }

    #[test]
    fn parse_new_tab() {
        let msg = r#"{"type":"NewTab"}"#;
        let parsed = parse_chrome_message(msg).unwrap();
        assert_eq!(parsed, ChromeToApp::NewTab);
    }

    #[test]
    fn parse_close_tab() {
        let msg = r#"{"type":"CloseTab","id":42}"#;
        let parsed = parse_chrome_message(msg).unwrap();
        assert_eq!(parsed, ChromeToApp::CloseTab { id: 42 });
    }

    #[test]
    fn parse_switch_tab() {
        let msg = r#"{"type":"SwitchTab","id":1}"#;
        let parsed = parse_chrome_message(msg).unwrap();
        assert_eq!(parsed, ChromeToApp::SwitchTab { id: 1 });
    }

    #[test]
    fn parse_reorder_tab() {
        let msg = r#"{"type":"ReorderTab","from":0,"to":2}"#;
        let parsed = parse_chrome_message(msg).unwrap();
        assert_eq!(parsed, ChromeToApp::ReorderTab { from: 0, to: 2 });
    }

    #[test]
    fn parse_go_back() {
        let msg = r#"{"type":"GoBack"}"#;
        assert_eq!(parse_chrome_message(msg).unwrap(), ChromeToApp::GoBack);
    }

    #[test]
    fn parse_go_forward() {
        let msg = r#"{"type":"GoForward"}"#;
        assert_eq!(parse_chrome_message(msg).unwrap(), ChromeToApp::GoForward);
    }

    #[test]
    fn parse_reload() {
        let msg = r#"{"type":"Reload"}"#;
        assert_eq!(parse_chrome_message(msg).unwrap(), ChromeToApp::Reload);
    }

    #[test]
    fn tab_created_to_js() {
        let msg = AppToChrome::TabCreated {
            id: 1,
            title: "Test".to_string(),
            url: "https://example.com".to_string(),
        };
        let js = msg.to_js_call();
        assert!(js.starts_with("handleMessage("));
        assert!(js.contains(r#""type":"TabCreated""#));
    }

    #[test]
    fn parse_invalid_json_returns_error() {
        assert!(parse_chrome_message("not json").is_err());
    }
}
```

- [ ] **Step 2: Add module declaration to main.rs**

Add to `src/main.rs`:
```rust
mod ipc;
```

- [ ] **Step 3: Run tests to verify they pass**

Run: `cargo test --lib ipc`
Expected: All 10 tests PASS

- [ ] **Step 4: Commit**

```bash
git add src/ipc.rs src/main.rs
git commit -m "feat: add IPC message types with serialization tests"
```

---

### Task 5: Engine trait

**Files:**
- Create: `src/engine/mod.rs`
- Modify: `src/main.rs` (add `mod engine;`)

- [ ] **Step 1: Define the WebEngine trait**

```rust
// src/engine/mod.rs

pub mod wry_engine;

use crate::tab::TabId;

pub type EngineResult<T> = Result<T, Box<dyn std::error::Error>>;

/// Callback type for engine events reported back to the app
pub enum EngineEvent {
    TitleChanged(TabId, String),
    UrlChanged(TabId, String),
    LoadingChanged(TabId, bool),
}

pub trait WebEngine {
    fn create_webview(
        &mut self,
        tab_id: TabId,
        url: &str,
        bounds: wry::Rect,
    ) -> EngineResult<()>;

    fn close_webview(&mut self, tab_id: TabId) -> EngineResult<()>;

    fn navigate(&self, tab_id: TabId, url: &str) -> EngineResult<()>;

    fn go_back(&self, tab_id: TabId) -> EngineResult<()>;

    fn go_forward(&self, tab_id: TabId) -> EngineResult<()>;

    fn reload(&self, tab_id: TabId) -> EngineResult<()>;

    fn set_visible(&self, tab_id: TabId, visible: bool) -> EngineResult<()>;

    fn set_bounds(&self, tab_id: TabId, bounds: wry::Rect) -> EngineResult<()>;
}
```

- [ ] **Step 2: Add module declaration to main.rs**

Add to `src/main.rs`:
```rust
mod engine;
```

- [ ] **Step 3: Create placeholder wry_engine.rs so it compiles**

```rust
// src/engine/wry_engine.rs
// Implemented in Task 8
```

- [ ] **Step 4: Run cargo check**

Run: `cargo check`
Expected: Compiles with no errors

- [ ] **Step 5: Commit**

```bash
git add src/engine/mod.rs src/engine/wry_engine.rs src/main.rs
git commit -m "feat: define WebEngine trait"
```

---

### Task 6: Keyboard shortcut detection

**Files:**
- Create: `src/keys.rs`
- Modify: `src/main.rs` (add `mod keys;`)

- [ ] **Step 1: Write keyboard shortcut module with tests**

```rust
// src/keys.rs

use winit::event::{ElementState, KeyEvent};
use winit::keyboard::{Key, ModifiersState, NamedKey};

#[derive(Debug, PartialEq)]
pub enum Shortcut {
    NewTab,
    CloseTab,
    FocusAddressBar,
    Reload,
}

pub fn detect_shortcut(modifiers: &ModifiersState, event: &KeyEvent) -> Option<Shortcut> {
    if event.state != ElementState::Pressed {
        return None;
    }

    let cmd_or_ctrl = if cfg!(target_os = "macos") {
        modifiers.super_key()
    } else {
        modifiers.control_key()
    };

    if !cmd_or_ctrl {
        return None;
    }

    match &event.logical_key {
        Key::Character(c) => match c.as_str() {
            "t" => Some(Shortcut::NewTab),
            "w" => Some(Shortcut::CloseTab),
            "l" => Some(Shortcut::FocusAddressBar),
            "r" => Some(Shortcut::Reload),
            _ => None,
        },
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use winit::event::ElementState;
    use winit::keyboard::{Key, PhysicalKey, NativeKeyCode};

    fn make_key_event(key: &str, state: ElementState) -> KeyEvent {
        KeyEvent {
            physical_key: PhysicalKey::Unidentified(NativeKeyCode::Unidentified),
            logical_key: Key::Character(key.into()),
            text: None,
            location: winit::keyboard::KeyLocation::Standard,
            state,
            repeat: false,
            platform_specific: Default::default(),
        }
    }

    fn cmd_modifiers() -> ModifiersState {
        if cfg!(target_os = "macos") {
            ModifiersState::SUPER
        } else {
            ModifiersState::CONTROL
        }
    }

    #[test]
    fn detect_new_tab() {
        let mods = cmd_modifiers();
        let event = make_key_event("t", ElementState::Pressed);
        assert_eq!(detect_shortcut(&mods, &event), Some(Shortcut::NewTab));
    }

    #[test]
    fn detect_close_tab() {
        let mods = cmd_modifiers();
        let event = make_key_event("w", ElementState::Pressed);
        assert_eq!(detect_shortcut(&mods, &event), Some(Shortcut::CloseTab));
    }

    #[test]
    fn detect_focus_address_bar() {
        let mods = cmd_modifiers();
        let event = make_key_event("l", ElementState::Pressed);
        assert_eq!(detect_shortcut(&mods, &event), Some(Shortcut::FocusAddressBar));
    }

    #[test]
    fn detect_reload() {
        let mods = cmd_modifiers();
        let event = make_key_event("r", ElementState::Pressed);
        assert_eq!(detect_shortcut(&mods, &event), Some(Shortcut::Reload));
    }

    #[test]
    fn no_shortcut_without_modifier() {
        let mods = ModifiersState::empty();
        let event = make_key_event("t", ElementState::Pressed);
        assert_eq!(detect_shortcut(&mods, &event), None);
    }

    #[test]
    fn no_shortcut_on_release() {
        let mods = cmd_modifiers();
        let event = make_key_event("t", ElementState::Released);
        assert_eq!(detect_shortcut(&mods, &event), None);
    }

    #[test]
    fn no_shortcut_for_unknown_key() {
        let mods = cmd_modifiers();
        let event = make_key_event("x", ElementState::Pressed);
        assert_eq!(detect_shortcut(&mods, &event), None);
    }
}
```

- [ ] **Step 2: Add module declaration to main.rs**

Add to `src/main.rs`:
```rust
mod keys;
```

- [ ] **Step 3: Run tests**

Run: `cargo test --lib keys`
Expected: All 7 tests PASS

- [ ] **Step 4: Commit**

```bash
git add src/keys.rs src/main.rs
git commit -m "feat: add keyboard shortcut detection with tests"
```

---

### Task 7: Chrome UI (HTML/CSS/JS)

**Files:**
- Create: `src/chrome.rs`
- Modify: `src/main.rs` (add `mod chrome;`)

- [ ] **Step 1: Create the chrome HTML generator**

```rust
// src/chrome.rs

/// Returns the HTML string for the browser chrome (tab bar + nav bar).
/// This is loaded into a small webview at the top of the window.
/// It communicates with Rust via window.ipc.postMessage(JSON).
pub fn chrome_html() -> String {
    r##"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<style>
  * { margin: 0; padding: 0; box-sizing: border-box; }
  body {
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    font-size: 13px;
    background: #2b2b2b;
    color: #e0e0e0;
    user-select: none;
    overflow: hidden;
  }

  /* Tab bar */
  #tab-bar {
    display: flex;
    align-items: center;
    height: 34px;
    background: #1e1e1e;
    padding: 0 4px;
    gap: 2px;
  }
  .tab {
    display: flex;
    align-items: center;
    height: 28px;
    padding: 0 12px;
    background: #2b2b2b;
    border-radius: 6px 6px 0 0;
    cursor: pointer;
    max-width: 200px;
    min-width: 60px;
    font-size: 12px;
    color: #999;
    transition: background 0.15s;
  }
  .tab:hover { background: #333; }
  .tab.active { background: #3c3c3c; color: #fff; }
  .tab-title {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
  }
  .tab-close {
    margin-left: 6px;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 11px;
    color: #888;
    cursor: pointer;
  }
  .tab-close:hover { background: #555; color: #fff; }
  #new-tab-btn {
    width: 28px;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 6px;
    cursor: pointer;
    font-size: 16px;
    color: #888;
  }
  #new-tab-btn:hover { background: #333; color: #fff; }

  /* Nav bar */
  #nav-bar {
    display: flex;
    align-items: center;
    height: 36px;
    background: #2b2b2b;
    padding: 0 8px;
    gap: 4px;
    border-top: 1px solid #3c3c3c;
  }
  .nav-btn {
    width: 28px;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 6px;
    cursor: pointer;
    font-size: 14px;
    color: #888;
    background: none;
    border: none;
  }
  .nav-btn:hover { background: #3c3c3c; color: #fff; }
  #address-bar {
    flex: 1;
    height: 26px;
    background: #1e1e1e;
    border: 1px solid #3c3c3c;
    border-radius: 6px;
    padding: 0 10px;
    color: #e0e0e0;
    font-size: 13px;
    outline: none;
  }
  #address-bar:focus { border-color: #5b9bd5; }
</style>
</head>
<body>

<div id="tab-bar">
  <div id="tabs-container"></div>
  <div id="new-tab-btn" onclick="send({type:'NewTab'})">+</div>
</div>

<div id="nav-bar">
  <button class="nav-btn" onclick="send({type:'GoBack'})">&#9664;</button>
  <button class="nav-btn" onclick="send({type:'GoForward'})">&#9654;</button>
  <button class="nav-btn" onclick="send({type:'Reload'})">&#8635;</button>
  <input id="address-bar" type="text" spellcheck="false"
         onkeydown="if(event.key==='Enter'){send({type:'Navigate',url:this.value})}">
</div>

<script>
  let tabs = [];
  let activeId = null;
  let dragSrcIdx = null;

  function send(msg) {
    window.ipc.postMessage(JSON.stringify(msg));
  }

  function renderTabs() {
    const container = document.getElementById('tabs-container');
    container.innerHTML = '';
    tabs.forEach((tab, idx) => {
      const el = document.createElement('div');
      el.className = 'tab' + (tab.id === activeId ? ' active' : '');
      el.draggable = true;
      el.innerHTML = '<span class="tab-title">' + escapeHtml(tab.title) + '</span>'
                   + '<span class="tab-close" onclick="event.stopPropagation();send({type:\'CloseTab\',id:' + tab.id + '})">&#215;</span>';
      el.onclick = () => send({type:'SwitchTab', id: tab.id});
      el.ondragstart = (e) => { dragSrcIdx = idx; e.dataTransfer.effectAllowed = 'move'; };
      el.ondragover = (e) => e.preventDefault();
      el.ondrop = (e) => {
        e.preventDefault();
        if (dragSrcIdx !== null && dragSrcIdx !== idx) {
          send({type:'ReorderTab', from: dragSrcIdx, to: idx});
        }
        dragSrcIdx = null;
      };
      container.appendChild(el);
    });
  }

  function escapeHtml(s) {
    const d = document.createElement('div');
    d.textContent = s;
    return d.innerHTML;
  }

  function handleMessage(msg) {
    switch (msg.type) {
      case 'TabCreated':
        tabs.push({id: msg.id, title: msg.title, url: msg.url, is_loading: false});
        activeId = msg.id;
        renderTabs();
        document.getElementById('address-bar').value = msg.url;
        break;
      case 'TabClosed':
        tabs = tabs.filter(t => t.id !== msg.id);
        renderTabs();
        break;
      case 'TabUpdated':
        tabs = tabs.map(t => t.id === msg.id ? {...t, title: msg.title, url: msg.url, is_loading: msg.is_loading} : t);
        renderTabs();
        if (msg.id === activeId) {
          document.getElementById('address-bar').value = msg.url;
        }
        break;
      case 'ActiveTabChanged':
        activeId = msg.id;
        renderTabs();
        const at = tabs.find(t => t.id === msg.id);
        if (at) document.getElementById('address-bar').value = at.url;
        break;
      case 'AllTabs':
        tabs = msg.tabs;
        activeId = msg.active_id;
        renderTabs();
        const act = tabs.find(t => t.id === activeId);
        if (act) document.getElementById('address-bar').value = act.url;
        break;
      case 'FocusAddressBar':
        document.getElementById('address-bar').focus();
        document.getElementById('address-bar').select();
        break;
    }
  }
</script>

</body>
</html>"##.to_string()
}
```

- [ ] **Step 2: Add module declaration to main.rs**

Add to `src/main.rs`:
```rust
mod chrome;
```

- [ ] **Step 3: Run cargo check**

Run: `cargo check`
Expected: Compiles with no errors

- [ ] **Step 4: Commit**

```bash
git add src/chrome.rs src/main.rs
git commit -m "feat: add browser chrome HTML/CSS/JS"
```

---

### Task 8: WryEngine implementation

**Files:**
- Modify: `src/engine/wry_engine.rs`

- [ ] **Step 1: Implement WryEngine**

```rust
// src/engine/wry_engine.rs

use std::collections::HashMap;
use winit::window::Window;
use wry::{Rect, WebView, WebViewBuilder};

use crate::engine::{EngineResult, WebEngine};
use crate::tab::TabId;

pub struct WryEngine<'a> {
    window: &'a Window,
    webviews: HashMap<TabId, WebView>,
}

impl<'a> WryEngine<'a> {
    pub fn new(window: &'a Window) -> Self {
        Self {
            window,
            webviews: HashMap::new(),
        }
    }
}

impl<'a> WebEngine for WryEngine<'a> {
    fn create_webview(
        &mut self,
        tab_id: TabId,
        url: &str,
        bounds: Rect,
    ) -> EngineResult<()> {
        let webview = WebViewBuilder::new()
            .with_bounds(bounds)
            .with_url(url)
            .build_as_child(self.window)?;
        self.webviews.insert(tab_id, webview);
        Ok(())
    }

    fn close_webview(&mut self, tab_id: TabId) -> EngineResult<()> {
        self.webviews.remove(&tab_id);
        Ok(())
    }

    fn navigate(&self, tab_id: TabId, url: &str) -> EngineResult<()> {
        if let Some(wv) = self.webviews.get(&tab_id) {
            wv.load_url(url)?;
        }
        Ok(())
    }

    fn go_back(&self, tab_id: TabId) -> EngineResult<()> {
        if let Some(wv) = self.webviews.get(&tab_id) {
            wv.evaluate_script("history.back()")?;
        }
        Ok(())
    }

    fn go_forward(&self, tab_id: TabId) -> EngineResult<()> {
        if let Some(wv) = self.webviews.get(&tab_id) {
            wv.evaluate_script("history.forward()")?;
        }
        Ok(())
    }

    fn reload(&self, tab_id: TabId) -> EngineResult<()> {
        if let Some(wv) = self.webviews.get(&tab_id) {
            wv.reload()?;
        }
        Ok(())
    }

    fn set_visible(&self, tab_id: TabId, visible: bool) -> EngineResult<()> {
        if let Some(wv) = self.webviews.get(&tab_id) {
            wv.set_visible(visible)?;
        }
        Ok(())
    }

    fn set_bounds(&self, tab_id: TabId, bounds: Rect) -> EngineResult<()> {
        if let Some(wv) = self.webviews.get(&tab_id) {
            wv.set_bounds(bounds)?;
        }
        Ok(())
    }
}
```

- [ ] **Step 2: Run cargo check**

Run: `cargo check`
Expected: Compiles. May need minor type adjustments based on exact wry API — fix any errors.

- [ ] **Step 3: Commit**

```bash
git add src/engine/wry_engine.rs
git commit -m "feat: implement WryEngine"
```

---

### Task 9: App main loop — wire everything together

**Files:**
- Modify: `src/main.rs`
- Modify: `src/app.rs` (create)

This is the largest task. It wires the chrome webview, engine, tab manager, keyboard shortcuts, and IPC together.

- [ ] **Step 1: Create app.rs with the full App struct and ApplicationHandler**

```rust
// src/app.rs

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    keyboard::ModifiersState,
    window::{Window, WindowId},
};
use wry::{
    dpi::{LogicalPosition, LogicalSize},
    Rect, WebView, WebViewBuilder,
};

use crate::chrome::chrome_html;
use crate::engine::wry_engine::WryEngine;
use crate::engine::WebEngine;
use crate::ipc::{self, AppToChrome, TabInfo};
use crate::keys::{self, Shortcut};
use crate::tab::{TabId, TabManager};
use crate::url::normalize_url;

const CHROME_HEIGHT: u32 = 70;
const DEFAULT_URL: &str = "about:blank";

pub struct App {
    window: Option<Window>,
    chrome_webview: Option<WebView>,
    engine: Option<WryEngine<'static>>,
    tabs: TabManager,
    modifiers: ModifiersState,
}

impl Default for App {
    fn default() -> Self {
        Self {
            window: None,
            chrome_webview: None,
            engine: None,
            tabs: TabManager::new(),
            modifiers: ModifiersState::empty(),
        }
    }
}

impl App {
    fn content_bounds(&self) -> Rect {
        let size = self
            .window
            .as_ref()
            .unwrap()
            .inner_size()
            .to_logical::<u32>(self.window.as_ref().unwrap().scale_factor());
        Rect {
            position: LogicalPosition::new(0, CHROME_HEIGHT).into(),
            size: LogicalSize::new(size.width, size.height.saturating_sub(CHROME_HEIGHT)).into(),
        }
    }

    fn send_to_chrome(&self, msg: &AppToChrome) {
        if let Some(chrome) = &self.chrome_webview {
            let _ = chrome.evaluate_script(&msg.to_js_call());
        }
    }

    fn handle_ipc(&mut self, body: &str) {
        let Ok(msg) = ipc::parse_chrome_message(body) else {
            return;
        };

        match msg {
            ipc::ChromeToApp::Navigate { url } => {
                let url = normalize_url(&url);
                if let Some(id) = self.tabs.active_id() {
                    if let Some(engine) = &self.engine {
                        let _ = engine.navigate(id, &url);
                    }
                    self.tabs.update_url(id, url.clone());
                    self.send_to_chrome(&AppToChrome::TabUpdated {
                        id: id.0,
                        title: self.tabs.active_tab().map(|t| t.title.clone()).unwrap_or_default(),
                        url,
                        is_loading: true,
                    });
                }
            }
            ipc::ChromeToApp::NewTab => self.create_tab(DEFAULT_URL),
            ipc::ChromeToApp::CloseTab { id } => self.close_tab(TabId(id)),
            ipc::ChromeToApp::SwitchTab { id } => self.switch_tab(TabId(id)),
            ipc::ChromeToApp::GoBack => {
                if let (Some(id), Some(engine)) = (self.tabs.active_id(), &self.engine) {
                    let _ = engine.go_back(id);
                }
            }
            ipc::ChromeToApp::GoForward => {
                if let (Some(id), Some(engine)) = (self.tabs.active_id(), &self.engine) {
                    let _ = engine.go_forward(id);
                }
            }
            ipc::ChromeToApp::Reload => {
                if let (Some(id), Some(engine)) = (self.tabs.active_id(), &self.engine) {
                    let _ = engine.reload(id);
                }
            }
            ipc::ChromeToApp::ReorderTab { from, to } => {
                self.tabs.reorder(from, to);
                self.sync_all_tabs();
            }
        }
    }

    fn create_tab(&mut self, url: &str) {
        let old_active = self.tabs.active_id();
        let id = self.tabs.create_tab(url);
        let bounds = self.content_bounds();

        if let Some(engine) = &mut self.engine {
            let _ = engine.create_webview(id, url, bounds);
            // Hide the previous tab's webview
            if let Some(old_id) = old_active {
                let _ = engine.set_visible(old_id, false);
            }
        }

        self.send_to_chrome(&AppToChrome::TabCreated {
            id: id.0,
            title: "New Tab".to_string(),
            url: url.to_string(),
        });
    }

    fn close_tab(&mut self, id: TabId) {
        if let Some(engine) = &mut self.engine {
            let _ = engine.close_webview(id);
        }
        let new_active = self.tabs.close_tab(id);

        self.send_to_chrome(&AppToChrome::TabClosed { id: id.0 });

        if let Some(new_id) = new_active {
            if let Some(engine) = &self.engine {
                let _ = engine.set_visible(new_id, true);
            }
            self.send_to_chrome(&AppToChrome::ActiveTabChanged { id: new_id.0 });
        }
        // If no tabs remain, the window_event CloseRequested path will handle exit
    }

    fn switch_tab(&mut self, id: TabId) {
        let old_active = self.tabs.active_id();
        self.tabs.set_active(id);

        if let Some(engine) = &self.engine {
            if let Some(old_id) = old_active {
                let _ = engine.set_visible(old_id, false);
            }
            let _ = engine.set_visible(id, true);
        }

        self.send_to_chrome(&AppToChrome::ActiveTabChanged { id: id.0 });
    }

    fn sync_all_tabs(&self) {
        let tabs: Vec<TabInfo> = self
            .tabs
            .tabs()
            .iter()
            .map(|t| TabInfo {
                id: t.id.0,
                title: t.title.clone(),
                url: t.url.clone(),
                is_loading: t.is_loading,
            })
            .collect();
        let active_id = self.tabs.active_id().map(|id| id.0).unwrap_or(0);
        self.send_to_chrome(&AppToChrome::AllTabs { tabs, active_id });
    }

    fn resize_all_webviews(&self) {
        let bounds = self.content_bounds();
        if let Some(engine) = &self.engine {
            for tab in self.tabs.tabs() {
                let _ = engine.set_bounds(tab.id, bounds);
            }
        }
        // Also resize chrome webview
        if let Some(chrome) = &self.chrome_webview {
            let size = self
                .window
                .as_ref()
                .unwrap()
                .inner_size()
                .to_logical::<u32>(self.window.as_ref().unwrap().scale_factor());
            let _ = chrome.set_bounds(Rect {
                position: LogicalPosition::new(0, 0).into(),
                size: LogicalSize::new(size.width, CHROME_HEIGHT).into(),
            });
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attrs = Window::default_attributes()
            .with_title("Light")
            .with_inner_size(LogicalSize::new(1280u32, 800u32));
        let window = event_loop.create_window(attrs).unwrap();
        let size = window.inner_size().to_logical::<u32>(window.scale_factor());

        // Create chrome webview at the top
        let chrome = WebViewBuilder::new()
            .with_bounds(Rect {
                position: LogicalPosition::new(0, 0).into(),
                size: LogicalSize::new(size.width, CHROME_HEIGHT).into(),
            })
            .with_html(&chrome_html())
            .with_ipc_handler(|req| {
                // This closure can't directly call self.handle_ipc because of borrow rules.
                // We use a UserEvent approach — see main.rs for the proxy wiring.
                // For now, this is a placeholder that will be replaced in main.rs
                // when we set up the event loop with UserEvent.
                let _ = req;
            })
            .with_focused(false)
            .build_as_child(&window)
            .unwrap();

        self.chrome_webview = Some(chrome);

        // Leak the window reference so the engine can hold a 'static ref
        // This is safe because the window lives for the entire program
        let window_ref: &'static Window = Box::leak(Box::new(window));
        self.engine = Some(WryEngine::new(window_ref));
        self.window = Some(unsafe { std::ptr::read(window_ref) });

        // Open default tab
        self.create_tab("https://start.duckduckgo.com");
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(_) => self.resize_all_webviews(),
            WindowEvent::ModifiersChanged(mods) => {
                self.modifiers = mods.state();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let Some(shortcut) = keys::detect_shortcut(&self.modifiers, &event) {
                    match shortcut {
                        Shortcut::NewTab => self.create_tab(DEFAULT_URL),
                        Shortcut::CloseTab => {
                            if let Some(id) = self.tabs.active_id() {
                                self.close_tab(id);
                                if self.tabs.is_empty() {
                                    event_loop.exit();
                                }
                            }
                        }
                        Shortcut::FocusAddressBar => {
                            if let Some(chrome) = &self.chrome_webview {
                                let _ = chrome.evaluate_script(
                                    "handleMessage({type:'FocusAddressBar'})",
                                );
                            }
                        }
                        Shortcut::Reload => {
                            if let (Some(id), Some(engine)) =
                                (self.tabs.active_id(), &self.engine)
                            {
                                let _ = engine.reload(id);
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
}
```

- [ ] **Step 2: Update main.rs to use App from app.rs**

```rust
// src/main.rs

mod app;
mod chrome;
mod engine;
mod ipc;
mod keys;
mod tab;
mod url;

use winit::event_loop::EventLoop;

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let mut app = app::App::default();
    event_loop.run_app(&mut app).unwrap();
}
```

- [ ] **Step 3: Build and fix compilation errors**

Run: `cargo build`
Expected: May need adjustments to lifetime handling and IPC wiring. Fix any errors.

The trickiest part is the IPC handler closure — it needs to send messages back to the app. The standard approach is to use `winit::event_loop::EventLoopProxy` with a custom `UserEvent`. If the closure-based IPC handler can't directly mutate App state (which it can't), refactor main.rs to:

1. Use `EventLoop::with_user_event()` if wry supports it, or
2. Use a channel (`std::sync::mpsc`) — the IPC handler sends on the channel, and the event loop polls it.

Use the channel approach since it's simpler with wry:

In `app.rs`, add a field:
```rust
pub ipc_receiver: Option<std::sync::mpsc::Receiver<String>>,
```

In the chrome webview creation, pass a sender clone to the IPC handler:
```rust
let (tx, rx) = std::sync::mpsc::channel();
self.ipc_receiver = Some(rx);
// ... in with_ipc_handler:
.with_ipc_handler(move |req| { let _ = tx.send(req.body().clone()); })
```

In `window_event`, after all other events, drain the channel:
```rust
WindowEvent::RedrawRequested => {
    while let Ok(body) = self.ipc_receiver.as_ref().unwrap().try_recv() {
        self.handle_ipc(&body);
    }
    self.window.as_ref().unwrap().request_redraw();
}
```

- [ ] **Step 4: Test manually**

Run: `cargo run`
Expected: Window opens with chrome bar at top, one tab with DuckDuckGo loaded. Can type URLs, open/close tabs, navigate back/forward/reload, use keyboard shortcuts, drag-reorder tabs.

- [ ] **Step 5: Commit**

```bash
git add src/app.rs src/main.rs
git commit -m "feat: wire app main loop with chrome, engine, tabs, and shortcuts"
```

---

### Task 10: Polish and cleanup

- [ ] **Step 1: Test all keyboard shortcuts manually**

- Cmd/Ctrl+T: opens new blank tab
- Cmd/Ctrl+W: closes current tab (quits if last tab)
- Cmd/Ctrl+L: focuses address bar
- Cmd/Ctrl+R: reloads current page

- [ ] **Step 2: Test tab drag-reorder**

Drag a tab to a new position. Tabs should reorder. Active tab should stay active.

- [ ] **Step 3: Run all unit tests**

Run: `cargo test`
Expected: All tests pass (url, tab, ipc, keys modules)

- [ ] **Step 4: Final commit**

```bash
git add -A
git commit -m "feat: Light browser v0.1.0 — minimal tabbed browser"
```
