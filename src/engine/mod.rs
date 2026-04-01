pub mod wry_engine;

use crate::tab::TabId;

pub type EngineResult<T> = Result<T, Box<dyn std::error::Error>>;

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
