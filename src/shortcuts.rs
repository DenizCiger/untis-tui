use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabId {
    Timetable,
    Absences,
}

#[derive(Debug, Clone)]
pub struct ShortcutSection {
    pub title: &'static str,
    pub items: Vec<ShortcutDisplay>,
}

#[derive(Debug, Clone)]
pub struct ShortcutDisplay {
    pub id: &'static str,
    pub keys: &'static str,
    pub action: &'static str,
}

fn char_key(key: KeyEvent, expected: char) -> bool {
    matches!(key.code, KeyCode::Char(value) if value == expected)
}

fn plain_char(key: KeyEvent, expected: char) -> bool {
    char_key(key, expected)
        && !key.modifiers.contains(KeyModifiers::CONTROL)
        && !key.modifiers.contains(KeyModifiers::ALT)
}

pub fn is_shortcut_pressed(id: &str, key: KeyEvent) -> bool {
    match id {
        "settings-open" => plain_char(key, '?'),
        "settings-close" => key.code == KeyCode::Esc || plain_char(key, '?'),
        "tab-prev" => plain_char(key, '['),
        "tab-next" => plain_char(key, ']'),
        "tab-timetable" => plain_char(key, '1'),
        "tab-absences" => plain_char(key, '2'),
        "quit" => plain_char(key, 'q'),
        "logout" => plain_char(key, 'l'),
        "timetable-week-prev" => {
            key.code == KeyCode::Left && key.modifiers.contains(KeyModifiers::SHIFT)
        }
        "timetable-week-next" => {
            key.code == KeyCode::Right && key.modifiers.contains(KeyModifiers::SHIFT)
        }
        "timetable-day-prev" => {
            key.code == KeyCode::Left && !key.modifiers.contains(KeyModifiers::SHIFT)
        }
        "timetable-day-next" => {
            key.code == KeyCode::Right && !key.modifiers.contains(KeyModifiers::SHIFT)
        }
        "timetable-up" => key.code == KeyCode::Up && !key.modifiers.contains(KeyModifiers::SHIFT),
        "timetable-down" => {
            key.code == KeyCode::Down && !key.modifiers.contains(KeyModifiers::SHIFT)
        }
        "timetable-up-step" => key.code == KeyCode::Up && key.modifiers.contains(KeyModifiers::SHIFT),
        "timetable-down-step" => {
            key.code == KeyCode::Down && key.modifiers.contains(KeyModifiers::SHIFT)
        }
        "timetable-page-up" => key.code == KeyCode::PageUp,
        "timetable-page-down" => key.code == KeyCode::PageDown,
        "timetable-home" => key.code == KeyCode::Home,
        "timetable-end" => key.code == KeyCode::End,
        "timetable-cycle-overlap" => key.code == KeyCode::Tab,
        "timetable-today" => plain_char(key, 't'),
        "timetable-refresh" => plain_char(key, 'r'),
        "timetable-search" => plain_char(key, '/'),
        "timetable-target-clear" => plain_char(key, 'c'),
        "timetable-search-up" => key.code == KeyCode::Up,
        "timetable-search-down" => key.code == KeyCode::Down,
        "timetable-search-submit" => key.code == KeyCode::Enter,
        "timetable-search-cancel" => key.code == KeyCode::Esc,
        "absences-up" => key.code == KeyCode::Up || plain_char(key, 'k'),
        "absences-down" => key.code == KeyCode::Down || plain_char(key, 'j'),
        "absences-page-up" => key.code == KeyCode::PageUp,
        "absences-page-down" => key.code == KeyCode::PageDown,
        "absences-home" => key.code == KeyCode::Home,
        "absences-end" => key.code == KeyCode::End,
        "absences-status" => plain_char(key, 'f'),
        "absences-window" => plain_char(key, 'w'),
        "absences-search" => plain_char(key, '/'),
        "absences-clear" => plain_char(key, 'c'),
        "absences-load-more" => plain_char(key, 'm'),
        "absences-refresh" => plain_char(key, 'r'),
        "absences-search-submit" => key.code == KeyCode::Enter,
        "absences-search-cancel" => key.code == KeyCode::Esc,
        "login-saved" => char_key(key, 'l') && key.modifiers.contains(KeyModifiers::CONTROL),
        "login-toggle-password" => {
            char_key(key, 'v') && key.modifiers.contains(KeyModifiers::CONTROL)
        }
        _ => false,
    }
}

fn display(id: &'static str, keys: &'static str, action: &'static str) -> ShortcutDisplay {
    ShortcutDisplay { id, keys, action }
}

fn pick(ids: &[&'static str]) -> Vec<ShortcutDisplay> {
    ids.iter()
        .map(|id| match *id {
            "settings-open" => display(id, "?", "Open shortcuts/settings"),
            "settings-close" => display(id, "Esc or ?", "Close settings modal"),
            "tab-prev" => display(id, "[", "Previous tab"),
            "tab-next" => display(id, "]", "Next tab"),
            "tab-timetable" => display(id, "1", "Jump to Timetable tab"),
            "tab-absences" => display(id, "2", "Jump to Absences tab"),
            "quit" => display(id, "q", "Quit app"),
            "logout" => display(id, "l", "Logout"),
            "timetable-week-prev" => display(id, "Shift+Left", "Previous week"),
            "timetable-week-next" => display(id, "Shift+Right", "Next week"),
            "timetable-day-prev" => display(id, "Left", "Move focus to previous day"),
            "timetable-day-next" => display(id, "Right", "Move focus to next day"),
            "timetable-up" => display(id, "Up", "Previous lesson period"),
            "timetable-down" => display(id, "Down", "Next lesson period"),
            "timetable-up-step" => display(id, "Shift+Up", "Move up one period"),
            "timetable-down-step" => display(id, "Shift+Down", "Move down one period"),
            "timetable-page-up" => display(id, "PageUp", "Jump up several periods"),
            "timetable-page-down" => display(id, "PageDown", "Jump down several periods"),
            "timetable-home" => display(id, "Home", "Jump to first lesson period"),
            "timetable-end" => display(id, "End", "Jump to last lesson period"),
            "timetable-cycle-overlap" => display(id, "Tab", "Cycle overlapping lessons"),
            "timetable-today" => display(id, "t", "Jump to current week/day"),
            "timetable-refresh" => display(id, "r", "Refresh timetable"),
            "timetable-search" => display(id, "/", "Open timetable target search"),
            "timetable-target-clear" => display(id, "c", "Switch back to own timetable"),
            "timetable-search-up" => display(id, "Up", "Move search highlight up"),
            "timetable-search-down" => display(id, "Down", "Move search highlight down"),
            "timetable-search-submit" => display(id, "Enter", "Apply highlighted timetable target"),
            "timetable-search-cancel" => display(id, "Esc", "Cancel timetable target search"),
            "absences-up" => display(id, "Up or k", "Move selection up"),
            "absences-down" => display(id, "Down or j", "Move selection down"),
            "absences-page-up" => display(id, "PageUp", "Jump one page up"),
            "absences-page-down" => display(id, "PageDown", "Jump one page down"),
            "absences-home" => display(id, "Home", "Jump to first record"),
            "absences-end" => display(id, "End", "Jump to last loaded record"),
            "absences-status" => display(id, "f", "Cycle status filter"),
            "absences-window" => display(id, "w", "Cycle time window"),
            "absences-search" => display(id, "/", "Open search"),
            "absences-clear" => display(id, "c", "Clear all filters"),
            "absences-load-more" => display(id, "m", "Load older records"),
            "absences-refresh" => display(id, "r", "Refresh absences"),
            "absences-search-submit" => display(id, "Enter", "Apply search query"),
            "absences-search-cancel" => display(id, "Esc", "Cancel search edit"),
            _ => display(id, "", ""),
        })
        .collect()
}

pub fn get_shortcut_sections(active_tab: TabId) -> Vec<ShortcutSection> {
    let mut sections = vec![
        ShortcutSection {
            title: "Global",
            items: pick(&[
                "settings-open",
                "tab-prev",
                "tab-next",
                "tab-timetable",
                "tab-absences",
                "logout",
                "quit",
            ]),
        },
        ShortcutSection {
            title: "Settings Modal",
            items: pick(&["settings-close"]),
        },
    ];

    match active_tab {
        TabId::Timetable => {
            sections.push(ShortcutSection {
                title: "Timetable",
                items: pick(&[
                    "timetable-week-prev",
                    "timetable-week-next",
                    "timetable-day-prev",
                    "timetable-day-next",
                    "timetable-up",
                    "timetable-down",
                    "timetable-up-step",
                    "timetable-down-step",
                    "timetable-page-up",
                    "timetable-page-down",
                    "timetable-home",
                    "timetable-end",
                    "timetable-cycle-overlap",
                    "timetable-today",
                    "timetable-refresh",
                    "timetable-search",
                    "timetable-target-clear",
                ]),
            });
            sections.push(ShortcutSection {
                title: "Timetable Search Input",
                items: pick(&[
                    "timetable-search-up",
                    "timetable-search-down",
                    "timetable-search-submit",
                    "timetable-search-cancel",
                ]),
            });
        }
        TabId::Absences => {
            sections.push(ShortcutSection {
                title: "Absences",
                items: pick(&[
                    "absences-up",
                    "absences-down",
                    "absences-page-up",
                    "absences-page-down",
                    "absences-home",
                    "absences-end",
                    "absences-status",
                    "absences-window",
                    "absences-search",
                    "absences-clear",
                    "absences-load-more",
                    "absences-refresh",
                ]),
            });
            sections.push(ShortcutSection {
                title: "Absences Search Input",
                items: pick(&["absences-search-submit", "absences-search-cancel"]),
            });
        }
    }

    sections
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent::new(code, modifiers)
    }

    #[test]
    fn shortcut_registry_matches_settings_open_shortcut() {
        assert!(is_shortcut_pressed(
            "settings-open",
            key(KeyCode::Char('?'), KeyModifiers::NONE)
        ));
    }

    #[test]
    fn shortcut_registry_requires_shift_for_previous_timetable_week() {
        assert!(is_shortcut_pressed(
            "timetable-week-prev",
            key(KeyCode::Left, KeyModifiers::SHIFT)
        ));
        assert!(!is_shortcut_pressed(
            "timetable-week-prev",
            key(KeyCode::Left, KeyModifiers::NONE)
        ));
    }

    #[test]
    fn shortcut_registry_uses_lesson_jump_bindings_for_timetable_vertical_nav() {
        assert!(is_shortcut_pressed("timetable-up", key(KeyCode::Up, KeyModifiers::NONE)));
        assert!(is_shortcut_pressed(
            "timetable-down",
            key(KeyCode::Down, KeyModifiers::NONE)
        ));
    }

    #[test]
    fn shortcut_registry_supports_timetable_paging_and_edge_shortcuts() {
        assert!(is_shortcut_pressed(
            "timetable-page-down",
            key(KeyCode::PageDown, KeyModifiers::NONE)
        ));
        assert!(is_shortcut_pressed("timetable-home", key(KeyCode::Home, KeyModifiers::NONE)));
    }

    #[test]
    fn shortcut_registry_supports_timetable_target_search_and_clear_shortcuts() {
        assert!(is_shortcut_pressed(
            "timetable-search",
            key(KeyCode::Char('/'), KeyModifiers::NONE)
        ));
        assert!(is_shortcut_pressed(
            "timetable-target-clear",
            key(KeyCode::Char('c'), KeyModifiers::NONE)
        ));
    }

    #[test]
    fn shortcut_registry_includes_contextual_sections_by_active_tab() {
        let timetable_sections = get_shortcut_sections(TabId::Timetable);
        let absences_sections = get_shortcut_sections(TabId::Absences);

        assert!(timetable_sections.iter().any(|section| section.title == "Timetable"));
        assert!(absences_sections.iter().any(|section| section.title == "Absences"));
    }
}
