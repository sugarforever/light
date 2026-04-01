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
use crate::chrome::chrome_html;
use crate::settings;
use crate::settings_page;
use crate::engine::wry_engine::WryEngine;
use crate::engine::WebEngine;
use crate::ipc::{self, AppToChrome, TabInfo};
use crate::keys::{self, Shortcut};
use crate::tab::{TabId, TabManager};
use crate::url::normalize_url;

const CHROME_HEIGHT: u32 = 98; // tab bar (36) + nav bar (34) + bookmarks bar (28)
const DEFAULT_URL: &str = "about:blank";

struct AppState {
    chrome_webview: Option<WebView>,
    engine: Option<WryEngine<'static>>,
    tabs: TabManager,
    bookmarks: Vec<Bookmark>,
    modifiers: ModifiersState,
    ipc_receiver: Option<mpsc::Receiver<String>>,
    window_width: u32,
    window_height: u32,
}

impl AppState {
    fn content_bounds(&self) -> Rect {
        Rect {
            position: LogicalPosition::new(0, CHROME_HEIGHT).into(),
            size: WryLogicalSize::new(self.window_width, self.window_height.saturating_sub(CHROME_HEIGHT)).into(),
        }
    }

    fn chrome_bounds(&self) -> Rect {
        Rect {
            position: LogicalPosition::new(0, 0).into(),
            size: WryLogicalSize::new(self.window_width, CHROME_HEIGHT).into(),
        }
    }

    fn send_to_chrome(&self, msg: &AppToChrome) {
        if let Some(chrome) = &self.chrome_webview {
            let _ = chrome.evaluate_script(&msg.to_js_call());
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
            ipc::ChromeToApp::AddBookmark { name, url } => {
                bookmarks::add(&mut self.bookmarks, &name, &url);
                self.sync_bookmarks();
            }
            ipc::ChromeToApp::RemoveBookmark { url } => {
                bookmarks::remove(&mut self.bookmarks, &url);
                self.sync_bookmarks();
            }
            ipc::ChromeToApp::ToggleBookmarksBar => {
                if let Some(chrome) = &self.chrome_webview {
                    let _ = chrome.evaluate_script(
                        "bookmarksBarVisible = !bookmarksBarVisible; \
                         document.getElementById('bookmarks-bar').classList.toggle('visible', bookmarksBarVisible && bookmarks.length > 0);"
                    );
                }
            }
            ipc::ChromeToApp::OpenSettings => {
                let s = settings::load();
                let html = settings_page::settings_html(&s.default_url);
                self.create_tab("light://settings");
                if let Some(id) = self.tabs.active_id() {
                    if let Some(engine) = &self.engine {
                        let _ = engine.load_html(id, &html);
                    }
                    self.tabs.update_title(id, "Settings".to_string());
                    self.send_to_chrome(&AppToChrome::TabUpdated {
                        id: id.0,
                        title: "Settings".to_string(),
                        url: "light://settings".to_string(),
                        is_loading: false,
                    });
                }
            }
            ipc::ChromeToApp::SaveSettings { default_url } => {
                let mut s = settings::load();
                s.default_url = default_url;
                settings::save(&s);
            }
            ipc::ChromeToApp::PageInfo { tab_id, title, url } => {
                let id = TabId(tab_id);
                self.tabs.update_title(id, title.clone());
                self.tabs.update_url(id, url.clone());
                self.send_to_chrome(&AppToChrome::TabUpdated {
                    id: tab_id,
                    title,
                    url,
                    is_loading: false,
                });
            }
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

        if let Some(engine) = &mut self.engine {
            let _ = engine.create_webview(id, url, bounds);
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
        let tabs: Vec<TabInfo> = self.tabs.tabs().iter().map(|t| TabInfo {
            id: t.id.0,
            title: t.title.clone(),
            url: t.url.clone(),
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
        if let Some(chrome) = &self.chrome_webview {
            let _ = chrome.set_bounds(self.chrome_bounds());
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

    let mut builder = WindowBuilder::new()
        .with_title("")
        .with_inner_size(LogicalSize::new(1280u32, 800u32));

    #[cfg(target_os = "macos")]
    {
        use tao::platform::macos::WindowBuilderExtMacOS;
        builder = builder
            .with_titlebar_transparent(true)
            .with_fullsize_content_view(true);
    }

    let window = builder
        .build(&event_loop)
        .unwrap();

    // Leak window for 'static reference
    let window: &'static tao::window::Window = Box::leak(Box::new(window));
    let size = window.inner_size().to_logical::<u32>(window.scale_factor());

    // Set up macOS Edit menu
    #[cfg(target_os = "macos")]
    setup_macos_edit_menu();

    // IPC channel
    let (tx, rx) = mpsc::channel::<String>();
    let engine_tx = tx.clone();

    // Chrome webview at the top (tao uses top-left origin)
    let chrome = WebViewBuilder::new()
        .with_bounds(Rect {
            position: LogicalPosition::new(0, 0).into(),
            size: WryLogicalSize::new(size.width, CHROME_HEIGHT).into(),
        })
        .with_html(&chrome_html())
        .with_ipc_handler(move |req| {
            let _ = tx.send(req.body().clone());
        })
        .with_focused(false)
        .build_as_child(window)
        .unwrap();

    let user_bookmarks = bookmarks::load();

    let mut state = AppState {
        chrome_webview: Some(chrome),
        engine: Some(WryEngine::new(window, engine_tx)),
        tabs: TabManager::new(),
        bookmarks: user_bookmarks,
        modifiers: ModifiersState::empty(),
        ipc_receiver: Some(rx),
        window_width: size.width,
        window_height: size.height,
    };

    // Open default tab from settings
    let user_settings = settings::load();
    state.create_tab(&user_settings.default_url);
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
                            Shortcut::NewTab => state.create_tab(DEFAULT_URL),
                            Shortcut::CloseTab => {
                                if let Some(id) = state.tabs.active_id() {
                                    state.close_tab(id);
                                    if state.tabs.is_empty() {
                                        *control_flow = ControlFlow::Exit;
                                    }
                                }
                            }
                            Shortcut::FocusAddressBar => {
                                if let Some(chrome) = &state.chrome_webview {
                                    let _ = chrome.evaluate_script(
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
