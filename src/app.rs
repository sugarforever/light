use std::sync::mpsc;
use tao::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::ModifiersState,
    window::WindowBuilder,
};
use wry::{
    dpi::{LogicalPosition, LogicalSize as WryLogicalSize},
    Rect, WebView, WebViewBuilder,
};

use crate::bookmarks::{self, Bookmark};
use crate::bookmarks_page;
use crate::chrome::chrome_html;
use crate::settings;
use crate::settings_page;
use crate::engine::wry_engine::WryEngine;
use crate::engine::WebEngine;
use crate::ipc::{self, AppToChrome, TabInfo};
use crate::keys::{self, Shortcut};
use crate::tab::{TabId, TabManager};
use crate::url::normalize_url;

const SIDEBAR_WIDTH_COMPACT: u32 = 44;
const SIDEBAR_WIDTH_EXPANDED: u32 = 220;
const NAV_BAR_HEIGHT: u32 = 38;
const FALLBACK_URL: &str = "about:blank";

struct AppState {
    window: &'static tao::window::Window,
    sidebar_webview: Option<WebView>,
    navbar_webview: Option<WebView>,
    engine: Option<WryEngine<'static>>,
    tabs: TabManager,
    bookmarks: Vec<Bookmark>,
    default_url: String,
    sidebar_width: u32,
    modifiers: ModifiersState,
    ipc_receiver: Option<mpsc::Receiver<String>>,
    window_width: u32,
    window_height: u32,
}

impl AppState {
    fn content_bounds(&self) -> Rect {
        Rect {
            position: LogicalPosition::new(self.sidebar_width, NAV_BAR_HEIGHT).into(),
            size: WryLogicalSize::new(
                self.window_width.saturating_sub(self.sidebar_width),
                self.window_height.saturating_sub(NAV_BAR_HEIGHT),
            ).into(),
        }
    }

    fn sidebar_bounds(&self) -> Rect {
        Rect {
            position: LogicalPosition::new(0, 0).into(),
            size: WryLogicalSize::new(self.sidebar_width, self.window_height).into(),
        }
    }

    fn navbar_bounds(&self) -> Rect {
        Rect {
            position: LogicalPosition::new(self.sidebar_width, 0).into(),
            size: WryLogicalSize::new(
                self.window_width.saturating_sub(self.sidebar_width),
                NAV_BAR_HEIGHT,
            ).into(),
        }
    }

    fn send_to_chrome(&self, msg: &AppToChrome) {
        let js = msg.to_js_call();
        if let Some(sidebar) = &self.sidebar_webview {
            let _ = sidebar.evaluate_script(&js);
        }
        if let Some(navbar) = &self.navbar_webview {
            let _ = navbar.evaluate_script(&js);
        }
    }

    fn drain_ipc(&mut self) {
        let Some(rx) = &self.ipc_receiver else { return };
        let mut messages = Vec::new();
        while let Ok(body) = rx.try_recv() {
            messages.push(body);
        }
        for body in messages {
            self.handle_ipc(&body);
        }
    }

    fn handle_ipc(&mut self, body: &str) {
        let Ok(msg) = ipc::parse_chrome_message(body) else { return };

        match msg {
            ipc::ChromeToApp::Navigate { url } => {
                let url = normalize_url(&url);
                if url.starts_with("light://") {
                    self.navigate_internal(&url);
                } else if let Some(id) = self.tabs.active_id() {
                    if let Some(engine) = &self.engine {
                        let _ = engine.navigate(id, &url);
                    }
                    self.tabs.update_url(id, url.clone());
                    self.send_to_chrome(&AppToChrome::TabUpdated {
                        id: id.0,
                        title: self.tabs.active_tab().map(|t| t.title.clone()).unwrap_or_default(),
                        url,
                        favicon: self.tabs.active_tab().map(|t| t.favicon.clone()).unwrap_or_default(),
                        is_loading: true,
                    });
                }
            }
            ipc::ChromeToApp::NewTab => {
                let url = self.default_url.clone();
                self.create_tab(&url);
            }
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
            ipc::ChromeToApp::AddBookmark { name, url } => {
                bookmarks::add(&mut self.bookmarks, &name, &url);
                self.sync_bookmarks();
            }
            ipc::ChromeToApp::RemoveBookmark { url } => {
                bookmarks::remove(&mut self.bookmarks, &url);
                self.sync_bookmarks();
            }
            ipc::ChromeToApp::ToggleBookmarksBar => {}
            ipc::ChromeToApp::OpenSettings => {
                self.create_tab("light://settings");
                self.navigate_internal("light://settings");
            }
            ipc::ChromeToApp::SaveSettings { default_url } => {
                let mut s = settings::load();
                s.default_url = default_url.clone();
                settings::save(&s);
                self.default_url = default_url;
            }
            ipc::ChromeToApp::ToggleSidebar => {
                self.sidebar_width = if self.sidebar_width == SIDEBAR_WIDTH_COMPACT {
                    SIDEBAR_WIDTH_EXPANDED
                } else {
                    SIDEBAR_WIDTH_COMPACT
                };
                let compact = self.sidebar_width == SIDEBAR_WIDTH_COMPACT;
                if let Some(sidebar) = &self.sidebar_webview {
                    let _ = sidebar.evaluate_script(&format!("setCompact({})", compact));
                }
                self.resize_all_webviews();
            }
            ipc::ChromeToApp::OpenUrl { url } => {
                self.create_tab(&url);
            }
            ipc::ChromeToApp::FocusAddressBar => {
                if let Some(navbar) = &self.navbar_webview {
                    let _ = navbar.evaluate_script("handleMessage({type:'FocusAddressBar'})");
                }
            }
            ipc::ChromeToApp::DragWindow => {
                let _ = self.window.drag_window();
            }
            ipc::ChromeToApp::PageInfo { tab_id, title, url, favicon } => {
                let id = TabId(tab_id);
                // Don't let page tracker overwrite internal pages
                let current_url = self.tabs.tabs().iter()
                    .find(|t| t.id == id)
                    .map(|t| t.url.clone())
                    .unwrap_or_default();
                if current_url.starts_with("light://") {
                    return;
                }
                self.tabs.update_title(id, title.clone());
                self.tabs.update_url(id, url.clone());
                if !favicon.is_empty() {
                    self.tabs.update_favicon(id, favicon.clone());
                }
                let fav = self.tabs.tabs().iter().find(|t| t.id == id).map(|t| t.favicon.clone()).unwrap_or_default();
                self.send_to_chrome(&AppToChrome::TabUpdated {
                    id: tab_id,
                    title,
                    url,
                    favicon: fav,
                    is_loading: false,
                });
                if self.tabs.active_id() == Some(id) {
                    self.update_window_title();
                }
            }
        }
    }

    fn navigate_internal(&mut self, url: &str) {
        match url {
            "light://settings" => {
                let s = settings::load();
                let html = settings_page::settings_html(&s.default_url);
                // Load in current tab or create new one
                if self.tabs.is_empty() {
                    self.create_tab(url);
                }
                if let Some(id) = self.tabs.active_id() {
                    if let Some(engine) = &self.engine {
                        let _ = engine.load_html(id, &html);
                    }
                    self.tabs.update_title(id, "Settings".to_string());
                    self.tabs.update_url(id, url.to_string());
                    self.send_to_chrome(&AppToChrome::TabUpdated {
                        id: id.0,
                        title: "Settings".to_string(),
                        url: url.to_string(),
                        favicon: String::new(),
                        is_loading: false,
                    });
                    self.update_window_title();
                }
            }
            "light://bookmarks" => {
                let html = bookmarks_page::bookmarks_html(&self.bookmarks);
                if self.tabs.is_empty() {
                    self.create_tab(url);
                }
                if let Some(id) = self.tabs.active_id() {
                    if let Some(engine) = &self.engine {
                        let _ = engine.load_html(id, &html);
                    }
                    self.tabs.update_title(id, "Bookmarks".to_string());
                    self.tabs.update_url(id, url.to_string());
                    self.send_to_chrome(&AppToChrome::TabUpdated {
                        id: id.0,
                        title: "Bookmarks".to_string(),
                        url: url.to_string(),
                        favicon: String::new(),
                        is_loading: false,
                    });
                    self.update_window_title();
                }
            }
            _ => {} // Unknown internal URI — ignore
        }
    }

    fn update_window_title(&self) {
        if let Some(tab) = self.tabs.active_tab() {
            let title = if tab.title.is_empty() || tab.title == "New Tab" {
                "Light".to_string()
            } else {
                tab.title.clone()
            };
            self.window.set_title(&title);
        }
    }

    fn sync_bookmarks(&self) {
        let bm: Vec<ipc::BookmarkInfo> = self.bookmarks.iter().map(|b| ipc::BookmarkInfo {
            name: b.name.clone(),
            url: b.url.clone(),
        }).collect();
        self.send_to_chrome(&AppToChrome::Bookmarks { bookmarks: bm });
    }

    fn create_tab(&mut self, url: &str) {
        let old_active = self.tabs.active_id();
        let id = self.tabs.create_tab(url);
        let bounds = self.content_bounds();

        // For internal URIs, create webview with about:blank — content loaded later via navigate_internal
        let webview_url = if url.starts_with("light://") { "about:blank" } else { url };

        if let Some(engine) = &mut self.engine {
            let _ = engine.create_webview(id, webview_url, bounds);
            if let Some(old_id) = old_active {
                let _ = engine.set_visible(old_id, false);
            }
        }

        self.send_to_chrome(&AppToChrome::TabCreated {
            id: id.0,
            title: "New Tab".to_string(),
            url: url.to_string(),
            favicon: String::new(),
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
        self.update_window_title();
    }

    fn sync_all_tabs(&self) {
        let tabs: Vec<TabInfo> = self.tabs.tabs().iter().map(|t| TabInfo {
            id: t.id.0,
            title: t.title.clone(),
            url: t.url.clone(),
            favicon: t.favicon.clone(),
            is_loading: t.is_loading,
        }).collect();
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
        if let Some(sidebar) = &self.sidebar_webview {
            let _ = sidebar.set_bounds(self.sidebar_bounds());
        }
        if let Some(navbar) = &self.navbar_webview {
            let _ = navbar.set_bounds(self.navbar_bounds());
        }
    }
}

#[cfg(target_os = "macos")]
fn setup_macos_icon() {
    // Skip if running from a .app bundle — the .icns handles the icon
    let exe = std::env::current_exe().unwrap_or_default();
    if exe.to_string_lossy().contains(".app/Contents/MacOS") {
        return;
    }

    // Only set icon when running as a raw binary (cargo run)
    use objc2_app_kit::NSApp;
    use objc2_foundation::{MainThreadMarker, NSData};

    static ICON_PNG: &[u8] = include_bytes!("../assets/AppIcon-256.png");

    let mtm = unsafe { MainThreadMarker::new_unchecked() };
    unsafe {
        let data = NSData::with_bytes(ICON_PNG);
        let cls = objc2::runtime::AnyClass::get("NSImage").unwrap();
        let image: *mut objc2::runtime::AnyObject = objc2::msg_send![cls, alloc];
        let image: *mut objc2::runtime::AnyObject = objc2::msg_send![image, initWithData: &*data];
        if !image.is_null() {
            let app = NSApp(mtm);
            let _: () = objc2::msg_send![&*app, setApplicationIconImage: image];
        }
    }
}

#[cfg(target_os = "macos")]
fn setup_macos_edit_menu() {
    use objc2_app_kit::{NSApp, NSMenu, NSMenuItem};
    use objc2_foundation::{MainThreadMarker, NSString};

    let mtm = unsafe { MainThreadMarker::new_unchecked() };
    unsafe {
        let app = NSApp(mtm);
        let menu_bar = NSMenu::new(mtm);

        // App menu (required as first item)
        let app_menu = NSMenu::new(mtm);
        let app_menu_item = NSMenuItem::new(mtm);
        app_menu_item.setSubmenu(Some(&app_menu));
        menu_bar.addItem(&app_menu_item);

        // Edit menu
        let edit_menu = NSMenu::new(mtm);
        edit_menu.setTitle(&NSString::from_str("Edit"));
        let edit_menu_item = NSMenuItem::new(mtm);
        edit_menu_item.setSubmenu(Some(&edit_menu));

        let make_item = |title: &str, action: objc2::runtime::Sel, key: &str| -> objc2::rc::Retained<NSMenuItem> {
            let item = NSMenuItem::new(mtm);
            item.setTitle(&NSString::from_str(title));
            item.setAction(Some(action));
            item.setKeyEquivalent(&NSString::from_str(key));
            item
        };

        edit_menu.addItem(&make_item("Undo", objc2::sel!(undo:), "z"));
        edit_menu.addItem(&make_item("Redo", objc2::sel!(redo:), "Z"));
        edit_menu.addItem(&NSMenuItem::separatorItem(mtm));
        edit_menu.addItem(&make_item("Cut", objc2::sel!(cut:), "x"));
        edit_menu.addItem(&make_item("Copy", objc2::sel!(copy:), "c"));
        edit_menu.addItem(&make_item("Paste", objc2::sel!(paste:), "v"));
        edit_menu.addItem(&make_item("Select All", objc2::sel!(selectAll:), "a"));

        menu_bar.addItem(&edit_menu_item);
        app.setMainMenu(Some(&menu_bar));
    }
}

pub fn run() {
    let event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_title("Light")
        .with_inner_size(LogicalSize::new(1280u32, 800u32))
        .build(&event_loop)
        .unwrap();

    // Leak window for 'static reference
    let window: &'static tao::window::Window = Box::leak(Box::new(window));
    let size = window.inner_size().to_logical::<u32>(window.scale_factor());

    // Set up macOS icon and Edit menu
    #[cfg(target_os = "macos")]
    {
        setup_macos_icon();
        setup_macos_edit_menu();
    }

    // IPC channel
    let (tx, rx) = mpsc::channel::<String>();
    let sidebar_tx = tx.clone();
    let navbar_tx = tx.clone();
    let engine_tx = tx;

    // Sidebar on the left (tabs) — starts compact
    let sidebar = WebViewBuilder::new()
        .with_bounds(Rect {
            position: LogicalPosition::new(0, 0).into(),
            size: WryLogicalSize::new(SIDEBAR_WIDTH_COMPACT, size.height).into(),
        })
        .with_html(&chrome_html())
        .with_ipc_handler(move |req| {
            let _ = sidebar_tx.send(req.body().clone());
        })
        .with_focused(false)
        .build_as_child(window)
        .unwrap();

    // Nav bar at the top of the content area
    let content_width = size.width.saturating_sub(SIDEBAR_WIDTH_COMPACT);
    let navbar = WebViewBuilder::new()
        .with_bounds(Rect {
            position: LogicalPosition::new(SIDEBAR_WIDTH_COMPACT, 0).into(),
            size: WryLogicalSize::new(content_width, NAV_BAR_HEIGHT).into(),
        })
        .with_html(&crate::navbar::navbar_html())
        .with_ipc_handler(move |req| {
            let _ = navbar_tx.send(req.body().clone());
        })
        .with_focused(false)
        .build_as_child(window)
        .unwrap();

    let user_bookmarks = bookmarks::load();
    let user_settings = settings::load();
    let default_url = user_settings.default_url.clone();

    let mut state = AppState {
        window,
        sidebar_webview: Some(sidebar),
        navbar_webview: Some(navbar),
        engine: Some(WryEngine::new(window, engine_tx)),
        tabs: TabManager::new(),
        bookmarks: user_bookmarks,
        default_url: default_url.clone(),
        sidebar_width: SIDEBAR_WIDTH_COMPACT,
        modifiers: ModifiersState::empty(),
        ipc_receiver: Some(rx),
        window_width: size.width,
        window_height: size.height,
    };

    // Open default tab from settings
    state.create_tab(&default_url);
    state.sync_bookmarks();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        // Drain IPC on every iteration
        state.drain_ipc();

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(physical_size) => {
                    let logical = physical_size.to_logical::<u32>(window.scale_factor());
                    state.window_width = logical.width;
                    state.window_height = logical.height;
                    state.resize_all_webviews();
                }
                WindowEvent::ModifiersChanged(mods) => {
                    state.modifiers = mods;
                }
                WindowEvent::KeyboardInput { event, .. } => {
                    if let Some(shortcut) = keys::detect_shortcut_tao(&state.modifiers, &event) {
                        match shortcut {
                            Shortcut::NewTab => {
                                let url = state.default_url.clone();
                                state.create_tab(&url);
                            }
                            Shortcut::CloseTab => {
                                if let Some(id) = state.tabs.active_id() {
                                    state.close_tab(id);
                                    if state.tabs.is_empty() {
                                        *control_flow = ControlFlow::Exit;
                                    }
                                }
                            }
                            Shortcut::FocusAddressBar => {
                                if let Some(navbar) = &state.navbar_webview {
                                    let _ = navbar.evaluate_script(
                                        "handleMessage({type:'FocusAddressBar'})",
                                    );
                                }
                            }
                            Shortcut::Reload => {
                                if let (Some(id), Some(engine)) =
                                    (state.tabs.active_id(), &state.engine)
                                {
                                    let _ = engine.reload(id);
                                }
                            }
                        }
                    }
                }
                _ => {}
            },
            _ => {}
        }
    });
}
