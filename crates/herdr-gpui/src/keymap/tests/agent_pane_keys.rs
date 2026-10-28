//! `[pane_keys.<agent>]` tables, which apply only while the focused pane runs
//! that agent.

use super::*;

fn with_agents(global: &[(&str, &str)], agents: &[(&str, &[(&str, &str)])]) -> Result<Keymap> {
    let mut table = pane_keys(global);
    for (agent, entries) in agents {
        let entries = entries
            .iter()
            .map(|(typed, sent)| ((*typed).to_owned(), (*sent).to_owned()))
            .collect();
        table.insert((*agent).to_owned(), PaneKeyEntry::Agent(entries));
    }
    Keymap::with_overrides(&BTreeMap::new(), &table, &no_keys())
}

fn text(text: &str) -> PaneKey {
    PaneKey::Text(text.into())
}

#[test]
fn an_agent_table_overrides_the_global_one_and_falls_through_to_it() {
    let keymap = with_agents(
        &[("cmd-enter", "text:global"), ("cmd-k", "ctrl-l")],
        &[(
            "claude",
            &[("enter", "text:\u{1b}[1;5F\r"), ("cmd-enter", "text:\r")],
        )],
    )
    .unwrap();
    let claude = |typed| keymap.agent_pane_key(Some("claude"), &keystroke(typed));
    assert_eq!(claude("enter"), Some(&text("\u{1b}[1;5F\r")));
    assert_eq!(claude("cmd-enter"), Some(&text("\r")));
    // Keystrokes the agent does not list fall through to the global table,
    // defaults included, then to normal input.
    assert_eq!(claude("cmd-k"), Some(&key("ctrl-l")));
    assert_eq!(claude("shift-enter"), Some(&key("alt-enter")));
    assert_eq!(claude("cmd-j"), None);
    // The plain lookup, used where no pane is in play, stays global.
    assert_eq!(sent(&keymap, "enter"), None);
    assert_eq!(sent(&keymap, "cmd-enter"), Some(&text("global")));
}

#[test]
fn a_pane_without_that_agent_ignores_its_table() {
    let keymap = with_agents(&[], &[("claude", &[("enter", "text:\r\r")])]).unwrap();
    for agent in [None, Some("codex"), Some("Claude")] {
        assert_eq!(keymap.agent_pane_key(agent, &keystroke("enter")), None);
        assert_eq!(
            keymap.agent_pane_key(agent, &keystroke("shift-enter")),
            Some(&key("alt-enter"))
        );
    }
    // A name no agent reports yet is accepted and never matches.
    assert!(with_agents(&[], &[("not-an-agent-yet", &[("cmd-enter", "text:\r")])]).is_ok());
}

#[test]
fn agent_tables_are_checked_like_the_global_one() {
    assert!(matches!(
        with_agents(&[], &[("claude", &[("7", "text:|")])]),
        Err(crate::Error::Keymap(Error::PaneKeyWithoutModifier(typed))) if typed == "claude.7"
    ));
    assert!(matches!(
        with_agents(&[], &[("claude", &[("cmd-enter", "text:")])]),
        Err(crate::Error::Keymap(Error::EmptyPaneKeyText(typed))) if typed == "claude.cmd-enter"
    ));
    let many: Vec<_> = (0..=MAX_PANE_KEYS)
        .map(|n| (format!("{}cmd-k", " ".repeat(n)), "ctrl-a".to_owned()))
        .collect();
    let many: Vec<_> = many
        .iter()
        .map(|(typed, sent)| (typed.as_str(), sent.as_str()))
        .collect();
    assert!(matches!(
        with_agents(&[], &[("claude", &many)]),
        Err(crate::Error::Keymap(Error::TooManyPaneKeys(MAX_PANE_KEYS)))
    ));
    // A configured command's keystroke stays the command's.
    let mut table = PaneKeys::new();
    table.insert(
        "claude".into(),
        PaneKeyEntry::Agent([("cmd-t".to_owned(), "ctrl-l".to_owned())].into()),
    );
    assert!(matches!(
        Keymap::with_overrides(&overrides(&[("new_tab", one("cmd-t"))]), &table, &no_keys()),
        Err(crate::Error::Keymap(Error::PaneKeyBound {
            command: "new_tab",
            ..
        }))
    ));
}

#[test]
fn agents_and_the_global_table_may_share_a_keystroke() {
    let keymap = with_agents(
        &[("cmd-enter", "text:a")],
        &[
            ("claude", &[("cmd-enter", "text:b")]),
            ("codex", &[("cmd-enter", "text:c")]),
        ],
    )
    .unwrap();
    let cmd_enter = keystroke("cmd-enter");
    assert_eq!(keymap.agent_pane_key(None, &cmd_enter), Some(&text("a")));
    assert_eq!(
        keymap.agent_pane_key(Some("claude"), &cmd_enter),
        Some(&text("b"))
    );
    assert_eq!(
        keymap.agent_pane_key(Some("codex"), &cmd_enter),
        Some(&text("c"))
    );
}
