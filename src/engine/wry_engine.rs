use std::collections::HashMap;
use std::sync::mpsc;
use tao::window::Window;
use wry::{Rect, WebView, WebViewBuilder};

use crate::engine::{EngineResult, WebEngine};
use crate::tab::TabId;

const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 14_0) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Safari/605.1.15";

pub struct WryEngine<'a> {
    window: &'a Window,
    webviews: HashMap<TabId, WebView>,
    ipc_sender: mpsc::Sender<String>,
}

impl<'a> WryEngine<'a> {
    pub fn new(window: &'a Window, ipc_sender: mpsc::Sender<String>) -> Self {
        Self {
            window,
            webviews: HashMap::new(),
            ipc_sender,
        }
    }
}

impl WebEngine for WryEngine<'_> {
    fn create_webview(
        &mut self,
        tab_id: TabId,
        url: &str,
        bounds: Rect,
    ) -> EngineResult<()> {
        let tx = self.ipc_sender.clone();
        let tab_id_val = tab_id.0;
        let webview = WebViewBuilder::new()
            .with_bounds(bounds)
            .with_url(url)
            .with_user_agent(USER_AGENT)
            .with_ipc_handler(move |req| {
                let _ = tx.send(req.body().clone());
            })
            .with_initialization_script(&format!(
                r#"
                (function() {{
                    // Page title/URL/favicon tracking (debounced)
                    let lastTitle = '';
                    let lastUrl = '';
                    let lastFavicon = '';
                    let debounceTimer = null;
                    function getFavicon() {{
                        var link = document.querySelector("link[rel~='icon']")
                            || document.querySelector("link[rel='shortcut icon']");
                        if (link) return link.href;
                        return location.origin + '/favicon.ico';
                    }}
                    function sendUpdate() {{
                        var fav = getFavicon();
                        if (document.title !== lastTitle || location.href !== lastUrl || fav !== lastFavicon) {{
                            lastTitle = document.title;
                            lastUrl = location.href;
                            lastFavicon = fav;
                            window.ipc.postMessage(JSON.stringify({{
                                type: 'PageInfo',
                                tab_id: {tab_id_val},
                                title: document.title,
                                url: location.href,
                                favicon: fav
                            }}));
                        }}
                    }}
                    function debouncedCheck() {{
                        if (debounceTimer) clearTimeout(debounceTimer);
                        debounceTimer = setTimeout(sendUpdate, 300);
                    }}
                    // Watch for title changes
                    var titleEl = document.querySelector('title');
                    if (titleEl) {{
                        new MutationObserver(debouncedCheck).observe(titleEl, {{childList: true, characterData: true, subtree: true}});
                    }}
                    // Watch for new <title> or <link rel=icon> being added
                    new MutationObserver(function(mutations) {{
                        for (var m of mutations) {{
                            for (var n of m.addedNodes) {{
                                if (n.tagName === 'TITLE' || (n.tagName === 'LINK' && n.rel && n.rel.includes('icon'))) {{
                                    debouncedCheck();
                                    if (n.tagName === 'TITLE') {{
                                        new MutationObserver(debouncedCheck).observe(n, {{childList: true, characterData: true, subtree: true}});
                                    }}
                                    return;
                                }}
                            }}
                        }}
                    }}).observe(document.head || document.documentElement, {{childList: true}});
                    // SPA navigation detection
                    window.addEventListener('popstate', debouncedCheck);
                    window.addEventListener('hashchange', debouncedCheck);
                    // Initial check
                    window.addEventListener('load', sendUpdate);
                    setTimeout(sendUpdate, 500);

                    // Keyboard shortcuts (Cmd/Ctrl + T/W/L/R)
                    document.addEventListener('keydown', function(e) {{
                        if (e.metaKey || e.ctrlKey) {{
                            var k = e.key.toLowerCase();
                            if (k === 't' || k === 'w' || k === 'l' || k === 'r') {{
                                e.preventDefault();
                                var msg = {{type: k === 't' ? 'NewTab' : k === 'w' ? 'CloseTab' : k === 'l' ? 'FocusAddressBar' : 'Reload'}};
                                if (k === 'w') msg.id = {tab_id_val};
                                window.ipc.postMessage(JSON.stringify(msg));
                            }}
                        }}
                    }});
                }})();
                "#
            ))
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

    fn load_html(&self, tab_id: TabId, html: &str) -> EngineResult<()> {
        if let Some(wv) = self.webviews.get(&tab_id) {
            wv.load_html(html)?;
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
