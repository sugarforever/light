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

impl WebEngine for WryEngine<'_> {
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
