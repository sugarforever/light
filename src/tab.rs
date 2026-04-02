#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TabId(pub u64);

#[derive(Debug, Clone)]
pub struct Tab {
    pub id: TabId,
    pub title: String,
    pub url: String,
    pub favicon: String,
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
        let id = TabId(self.next_id);
        self.next_id += 1;
        self.tabs.push(Tab {
            id,
            title: "New Tab".to_string(),
            url: url.to_string(),
            favicon: String::new(),
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

    pub fn update_favicon(&mut self, id: TabId, favicon: String) {
        if let Some(idx) = self.find_index(id) {
            self.tabs[idx].favicon = favicon;
        }
    }

    pub fn set_loading(&mut self, id: TabId, loading: bool) {
        if let Some(idx) = self.find_index(id) {
            self.tabs[idx].is_loading = loading;
        }
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
        mgr.create_tab("https://b.com");
        mgr.create_tab("https://c.com");
        mgr.set_active(id1);
        let new_active = mgr.close_tab(id1);
        assert!(new_active.is_some());
        assert!(mgr.active_tab().is_some());
    }

    #[test]
    fn set_active_changes_active_tab() {
        let mut mgr = TabManager::new();
        let id1 = mgr.create_tab("https://a.com");
        mgr.create_tab("https://b.com");
        mgr.set_active(id1);
        assert_eq!(mgr.active_id(), Some(id1));
    }

    #[test]
    fn reorder_moves_tab() {
        let mut mgr = TabManager::new();
        let id1 = mgr.create_tab("https://a.com");
        mgr.create_tab("https://b.com");
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
