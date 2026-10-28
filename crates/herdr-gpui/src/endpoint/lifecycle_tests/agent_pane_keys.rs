use super::*;

/// A pane running Claude gets `[pane_keys.claude]`; once Herdr reports the
/// pane back to a shell, Enter is the plain key again.
#[gpui::test]
fn agent_pane_keys_follow_the_focused_panes_agent(cx: &mut gpui::TestAppContext) {
    let (fixture, cx) = cx.add_window_view(|window, cx| {
        Fixture(cx.new(|cx| crate::sidebar::layout_tests::fixture_window(window, cx)))
    });
    let view = fixture.update(cx, |fixture, _| fixture.0.clone());
    let (endpoint, mut server) = connected_endpoint("agent-pane-keys");
    let down = |keystroke: &str| gpui::KeyDownEvent {
        keystroke: gpui::Keystroke::parse(keystroke).unwrap(),
        is_held: false,
        prefer_character_input: false,
    };
    let input = |event| ClientMessage::ClientShellPaneInput {
        pane_id: "w1:p1".into(),
        events: vec![event],
    };
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            prepare_mouse(view, endpoint, cx);
            view.live.activation = Some(crate::state::SurfaceActivation {
                request: "activate-1".into(),
                boot: view.live.snapshot.as_ref().unwrap().boot_id.clone(),
                revision: Some(view.live.surface.as_ref().unwrap().projection_revision),
                failed: false,
                focus: None,
                active: true,
            });
            let claude = [
                ("enter".to_owned(), "text:\u{1b}[1;5F\r".to_owned()),
                ("cmd-enter".to_owned(), "text:\r".to_owned()),
            ];
            view.config.keybindings = crate::keymap::Keymap::with_overrides(
                &Default::default(),
                &[(
                    "claude".into(),
                    crate::keymap::PaneKeyEntry::Agent(claude.into()),
                )]
                .into(),
                &crate::keymap::DaemonKeys::default(),
            )
            .unwrap();
            assert_eq!(view.focused_pane_agent(), Some("claude"));
            view.key_down(&down("enter"), window, cx);
            view.key_down(&down("cmd-enter"), window, cx);
            // The agent exits: Herdr no longer names one for the pane.
            Arc::make_mut(view.live.snapshot.as_mut().unwrap()).agents[0].agent = None;
            assert_eq!(view.focused_pane_agent(), None);
            view.key_down(&down("enter"), window, cx);
        });
    });
    assert_eq!(
        server.receive(),
        input(ClientPaneInputEvent::TextCommit("\u{1b}[1;5F\r".into()))
    );
    assert_eq!(
        server.receive(),
        input(ClientPaneInputEvent::TextCommit("\r".into()))
    );
    assert_eq!(
        server.receive(),
        input(ClientPaneInputEvent::Key {
            code: ClientKeyCode::Enter,
            modifiers: 0,
            kind: ClientKeyKind::Press,
            repeat_count: 1,
            shifted_codepoint: None,
            generated_text: None,
            tracks_release: false,
            physical_key_id: None,
            windows_record: None,
        })
    );
}
