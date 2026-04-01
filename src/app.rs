use std::sync::mpsc;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    keyboard::ModifiersState,
    window::{Window, WindowId},
};

#[cfg(target_os = "macos")]
use objc2_app_kit::{NSApp, NSMenu, NSMenuItem};
#[cfg(target_os = "macos")]
use objc2_foundation::{MainThreadMarker, NSString};
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
    window: Option<&'static Window>,
    chrome_webview: Option<WebView>,
    engine: Option<WryEngine<'static>>,
    tabs: TabManager,
    modifiers: ModifiersState,
    ipc_receiver: Option<mpsc::Receiver<String>>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            window: None,
            chrome_webview: None,
            engine: None,
            tabs: TabManager::new(),
            modifiers: ModifiersState::empty(),
            ipc_receiver: None,
        }
    }
}

impl App {
    fn window(&self) -> &Window {
        self.window.expect("window not initialized")
    }

    fn content_bounds(&self) -> Rect {
        let size = self
            .window()
            .inner_size()
            .to_logical::<u32>(self.window().scale_factor());
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

    fn drain_ipc(&mut self) {
        let Some(rx) = &self.ipc_receiver else {
            return;
        };
        let mut messages = Vec::new();
        while let Ok(body) = rx.try_recv() {
            messages.push(body);
        }
        for body in messages {
            self.handle_ipc(&body);
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
                        title: self
                            .tabs
                            .active_tab()
                            .map(|t| t.title.clone())
                            .unwrap_or_default(),
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
        if let Some(chrome) = &self.chrome_webview {
            let size = self
                .window()
                .inner_size()
                .to_logical::<u32>(self.window().scale_factor());
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

        // Leak the window so we get a 'static reference for the engine and chrome webview
        let window: &'static Window = Box::leak(Box::new(window));

        let size = window.inner_size().to_logical::<u32>(window.scale_factor());

        // Set up IPC channel
        let (tx, rx) = mpsc::channel::<String>();
        self.ipc_receiver = Some(rx);

        // Create chrome webview at the top
        let chrome = WebViewBuilder::new()
            .with_bounds(Rect {
                position: LogicalPosition::new(0, 0).into(),
                size: LogicalSize::new(size.width, CHROME_HEIGHT).into(),
            })
            .with_html(&chrome_html())
            .with_ipc_handler(move |req| {
                let _ = tx.send(req.body().clone());
            })
            .with_focused(false)
            .build_as_child(window)
            .unwrap();

        self.chrome_webview = Some(chrome);
        self.window = Some(window);
        self.engine = Some(WryEngine::new(window));

        // On macOS, add a native Edit menu so Cmd+A/C/V/X work in webviews
        #[cfg(target_os = "macos")]
        {
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

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        self.drain_ipc();
    }
}
