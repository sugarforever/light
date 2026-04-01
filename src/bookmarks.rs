use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Bookmark {
    pub name: String,
    pub url: String,
}

fn bookmarks_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("light")
        .join("bookmarks.json")
}

pub fn load() -> Vec<Bookmark> {
    let path = bookmarks_path();
    match fs::read_to_string(&path) {
        Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

pub fn save(bookmarks: &[Bookmark]) {
    let path = bookmarks_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let json = serde_json::to_string_pretty(bookmarks).unwrap();
    let _ = fs::write(path, json);
}

pub fn add(bookmarks: &mut Vec<Bookmark>, name: &str, url: &str) {
    if !bookmarks.iter().any(|b| b.url == url) {
        bookmarks.push(Bookmark {
            name: name.to_string(),
            url: url.to_string(),
        });
        save(bookmarks);
    }
}

pub fn remove(bookmarks: &mut Vec<Bookmark>, url: &str) {
    bookmarks.retain(|b| b.url != url);
    save(bookmarks);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_bookmark() {
        let mut bm = Vec::new();
        add(&mut bm, "Test", "https://example.com");
        assert_eq!(bm.len(), 1);
        assert_eq!(bm[0].name, "Test");
    }

    #[test]
    fn add_duplicate_is_noop() {
        let mut bm = Vec::new();
        add(&mut bm, "Test", "https://example.com");
        add(&mut bm, "Test 2", "https://example.com");
        assert_eq!(bm.len(), 1);
    }

    #[test]
    fn remove_bookmark() {
        let mut bm = vec![Bookmark {
            name: "Test".to_string(),
            url: "https://example.com".to_string(),
        }];
        remove(&mut bm, "https://example.com");
        assert!(bm.is_empty());
    }

    #[test]
    fn round_trip_serialize() {
        let bm = vec![Bookmark {
            name: "Test".to_string(),
            url: "https://example.com".to_string(),
        }];
        let json = serde_json::to_string(&bm).unwrap();
        let bm2: Vec<Bookmark> = serde_json::from_str(&json).unwrap();
        assert_eq!(bm, bm2);
    }
}
