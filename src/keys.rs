use winit::event::{ElementState, KeyEvent};
use winit::keyboard::{Key, ModifiersState};

#[derive(Debug, PartialEq)]
pub enum Shortcut {
    NewTab,
    CloseTab,
    FocusAddressBar,
    Reload,
}

fn match_shortcut(key: &Key, pressed: bool, cmd_or_ctrl: bool) -> Option<Shortcut> {
    if !pressed || !cmd_or_ctrl {
        return None;
    }

    match key {
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

pub fn detect_shortcut(modifiers: &ModifiersState, event: &KeyEvent) -> Option<Shortcut> {
    let pressed = event.state == ElementState::Pressed;
    let cmd_or_ctrl = if cfg!(target_os = "macos") {
        modifiers.super_key()
    } else {
        modifiers.control_key()
    };
    match_shortcut(&event.logical_key, pressed, cmd_or_ctrl)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_new_tab() {
        let key = Key::Character("t".into());
        assert_eq!(match_shortcut(&key, true, true), Some(Shortcut::NewTab));
    }

    #[test]
    fn detect_close_tab() {
        let key = Key::Character("w".into());
        assert_eq!(match_shortcut(&key, true, true), Some(Shortcut::CloseTab));
    }

    #[test]
    fn detect_focus_address_bar() {
        let key = Key::Character("l".into());
        assert_eq!(
            match_shortcut(&key, true, true),
            Some(Shortcut::FocusAddressBar)
        );
    }

    #[test]
    fn detect_reload() {
        let key = Key::Character("r".into());
        assert_eq!(match_shortcut(&key, true, true), Some(Shortcut::Reload));
    }

    #[test]
    fn no_shortcut_without_modifier() {
        let key = Key::Character("t".into());
        assert_eq!(match_shortcut(&key, true, false), None);
    }

    #[test]
    fn no_shortcut_on_release() {
        let key = Key::Character("t".into());
        assert_eq!(match_shortcut(&key, false, true), None);
    }

    #[test]
    fn no_shortcut_for_unknown_key() {
        let key = Key::Character("x".into());
        assert_eq!(match_shortcut(&key, true, true), None);
    }
}
