use super::*;
use crate::keymap::{PaneKey, PaneKeyEntry};

/// A string and a sub-table sit side by side in `[pane_keys]`: the string is
/// a global key, the sub-table an agent's own keys, and both may name the
/// same keystroke.
#[test]
fn global_keys_and_agent_tables_parse_together() -> anyhow::Result<()> {
    let config = Config::parse(concat!(
        "[pane_keys]\n",
        "\"cmd-enter\" = \"text:global\"\n",
        "\"cmd-k\" = \"ctrl-l\"\n",
        "[pane_keys.claude]\n",
        "enter = \"text:\\u001b[1;5F\\r\"\n",
        "\"cmd-enter\" = \"text:\\r\"\n",
    ))?;
    assert!(config.unknown_keys.is_empty(), "{:?}", config.unknown_keys);
    assert_eq!(
        config.pane_keys.get("cmd-enter"),
        Some(&PaneKeyEntry::Key("text:global".into()))
    );
    assert!(matches!(
        config.pane_keys.get("claude"),
        Some(PaneKeyEntry::Agent(table)) if table.len() == 2
    ));
    let sent = |agent, typed| {
        config
            .keybindings
            .agent_pane_key(agent, &gpui::Keystroke::parse(typed).unwrap_or_default())
            .cloned()
    };
    let text = |text: &str| Some(PaneKey::Text(text.into()));
    assert_eq!(sent(Some("claude"), "enter"), text("\u{1b}[1;5F\r"));
    assert_eq!(sent(Some("claude"), "cmd-enter"), text("\r"));
    assert_eq!(sent(None, "cmd-enter"), text("global"));
    assert_eq!(sent(None, "enter"), None);
    assert!(matches!(
        sent(Some("claude"), "cmd-k"),
        Some(PaneKey::Key(key)) if key.unparse() == "ctrl-l"
    ));
    Ok(())
}

#[test]
fn agent_tables_hold_only_keystrokes() {
    // An agent table nested a level deeper is neither a value nor a table of
    // values.
    assert!(Config::parse("[pane_keys.claude.enter]\nsent = \"text:\\r\"\n").is_err());
    assert!(Config::parse("[pane_keys.claude]\nenter = 1\n").is_err());
    assert!(matches!(
        Config::parse("[pane_keys.claude]\n\"7\" = \"text:|\"\n"),
        Err(crate::Error::Keymap(
            crate::keymap::Error::PaneKeyWithoutModifier(typed)
        )) if typed == "claude.7"
    ));
}
