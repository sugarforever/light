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
    AddBookmark { name: String, url: String },
    RemoveBookmark { url: String },
    ToggleBookmarksBar,
    OpenSettings,
    SaveSettings { default_url: String },
    PageInfo { tab_id: u64, title: String, url: String },
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
    Bookmarks { bookmarks: Vec<BookmarkInfo> },
}

#[derive(Debug, Serialize, PartialEq)]
pub struct BookmarkInfo {
    pub name: String,
    pub url: String,
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
        assert_eq!(
            parsed,
            ChromeToApp::Navigate {
                url: "https://example.com".to_string()
            }
        );
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
