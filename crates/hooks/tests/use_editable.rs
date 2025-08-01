use freya::prelude::*;
use freya_testing::prelude::*;

use crate::{
    use_editable,
    EditableMode,
    TextEditor,
};

#[tokio::test]
pub async fn multiple_lines_single_editor() {
    fn use_editable_app() -> Element {
        let mut editable = use_editable(
            || EditableConfig::new("Hello Rustaceans\nHello Rustaceans".to_string()),
            EditableMode::MultipleLinesSingleEditor,
        );
        let cursor_attr = editable.cursor_attr();
        let editor = editable.editor().read();
        let cursor_pos = editor.cursor_pos();

        let onmousedown = move |e: MouseEvent| {
            editable.process_event(&EditableEvent::MouseDown(e.data, 0));
        };

        let onglobalkeydown = move |e: Event<KeyboardData>| {
            editable.process_event(&EditableEvent::KeyDown(e.data));
        };

        rsx!(
            rect {
                width: "100%",
                height: "100%",
                background: "white",
                onmousedown,
                paragraph {
                    cursor_reference: cursor_attr,
                    height: "50%",
                    width: "100%",
                    cursor_id: "0",
                    cursor_index: "{cursor_pos}",
                    cursor_color: "black",
                    cursor_mode: "editable",
                    onglobalkeydown,
                    text {
                        color: "black",
                        "{editor}"
                    }
                }
                label {
                    color: "black",
                    height: "50%",
                    "{editor.cursor_row()}:{editor.cursor_col()}"
                }
            }
        )
    }

    let mut utils = launch_test(use_editable_app);

    // Initial state
    let root = utils.root().get(0);
    let cursor = root.get(1).get(0);
    let content = root.get(0).get(0).get(0);
    assert_eq!(cursor.text(), Some("0:0"));
    assert_eq!(content.text(), Some("Hello Rustaceans\nHello Rustaceans"));

    // Move cursor
    utils.push_event(TestEvent::Mouse {
        name: MouseEventName::MouseDown,
        cursor: (35.0, 3.0).into(),
        button: Some(MouseButton::Left),
    });

    utils.wait_for_update().await;
    utils.wait_for_update().await;

    // Cursor has been moved
    let root = utils.root().get(0);
    let cursor = root.get(1).get(0);
    assert_eq!(cursor.text(), Some("0:5"));

    // Insert text
    utils.push_event(TestEvent::Keyboard {
        name: KeyboardEventName::KeyDown,
        key: Key::Character("!".to_string()),
        code: Code::Unidentified,
        modifiers: Modifiers::empty(),
    });

    utils.wait_for_update().await;

    // Text and cursor have changed
    let cursor = root.get(1).get(0);
    let content = root.get(0).get(0).get(0);
    assert_eq!(content.text(), Some("Hello! Rustaceans\nHello Rustaceans"));
    assert_eq!(cursor.text(), Some("0:6"));

    // Move cursor to the begining
    utils.push_event(TestEvent::Mouse {
        name: MouseEventName::MouseDown,
        cursor: (3.0, 3.0).into(),
        button: Some(MouseButton::Left),
    });
    utils.wait_for_update().await;
    utils.wait_for_update().await;
    let cursor = root.get(1).get(0);
    assert_eq!(cursor.text(), Some("0:0"));

    // Move cursor with arrow down
    utils.push_event(TestEvent::Keyboard {
        name: KeyboardEventName::KeyDown,
        code: Code::ArrowDown,
        key: Key::ArrowDown,
        modifiers: Modifiers::default(),
    });
    utils.wait_for_update().await;
    let cursor = root.get(1).get(0);
    assert_eq!(cursor.text(), Some("1:0"));

    // Move cursor with arrow right
    utils.push_event(TestEvent::Keyboard {
        name: KeyboardEventName::KeyDown,
        code: Code::ArrowRight,
        key: Key::ArrowRight,
        modifiers: Modifiers::default(),
    });
    utils.wait_for_update().await;
    let cursor = root.get(1).get(0);
    assert_eq!(cursor.text(), Some("1:1"));

    // Move cursor with arrow up
    utils.push_event(TestEvent::Keyboard {
        name: KeyboardEventName::KeyDown,
        code: Code::ArrowUp,
        key: Key::ArrowUp,
        modifiers: Modifiers::default(),
    });
    utils.wait_for_update().await;
    let cursor = root.get(1).get(0);
    assert_eq!(cursor.text(), Some("0:1"));

    // Move cursor with arrow left
    utils.push_event(TestEvent::Keyboard {
        name: KeyboardEventName::KeyDown,
        code: Code::ArrowLeft,
        key: Key::ArrowLeft,
        modifiers: Modifiers::default(),
    });
    utils.wait_for_update().await;
    let cursor = root.get(1).get(0);
    assert_eq!(cursor.text(), Some("0:0"));

    // Move cursor with arrow down, twice
    utils.push_event(TestEvent::Keyboard {
        name: KeyboardEventName::KeyDown,
        code: Code::ArrowDown,
        key: Key::ArrowDown,
        modifiers: Modifiers::default(),
    });
    utils.push_event(TestEvent::Keyboard {
        name: KeyboardEventName::KeyDown,
        code: Code::ArrowDown,
        key: Key::ArrowDown,
        modifiers: Modifiers::default(),
    });
    utils.wait_for_update().await;
    let cursor = root.get(1).get(0);
    // Because there is not a third line, the cursor will be moved to the max right
    assert_eq!(cursor.text(), Some("1:16"));

    // Move cursor with arrow up, twice
    utils.push_event(TestEvent::Keyboard {
        name: KeyboardEventName::KeyDown,
        code: Code::ArrowUp,
        key: Key::ArrowUp,
        modifiers: Modifiers::default(),
    });
    utils.push_event(TestEvent::Keyboard {
        name: KeyboardEventName::KeyDown,
        code: Code::ArrowUp,
        key: Key::ArrowUp,
        modifiers: Modifiers::default(),
    });
    utils.wait_for_update().await;
    let cursor = root.get(1).get(0);
    // Because there is not a line above the first one, the cursor will be moved to the begining
    assert_eq!(cursor.text(), Some("0:0"));
}

#[tokio::test]
pub async fn single_line_multiple_editors() {
    fn use_editable_app() -> Element {
        let mut editable = use_editable(
            || EditableConfig::new("Hello Rustaceans\nHello World".to_string()),
            EditableMode::SingleLineMultipleEditors,
        );
        let editor = editable.editor().read();

        let onglobalkeydown = move |e: Event<KeyboardData>| {
            editable.process_event(&EditableEvent::KeyDown(e.data));
        };

        rsx!(
            rect {
                width: "100%",
                height: "100%",
                background: "white",
                onglobalkeydown,
                {editor.lines().enumerate().map(move |(i, line)| {

                    let onmousedown = move |e: MouseEvent| {
                        editable.process_event(&EditableEvent::MouseDown(e.data, 0));
                    };

                    rsx!(
                        paragraph {
                            cursor_reference: editable.cursor_attr(),
                            width: "100%",
                            height: "30",
                            max_lines: "1",
                            cursor_id: "0",
                            cursor_index: "{i}",
                            cursor_color: "black",
                            cursor_mode: "editable",
                            onmousedown,
                            text {
                                color: "black",
                                "{line}"
                            }
                        }
                    )
                })}
                label {
                    color: "black",
                    height: "50%",
                    "{editor.cursor_row()}:{editor.cursor_col()}"
                }
            }
        )
    }

    let mut utils = launch_test(use_editable_app);

    // Initial state
    let root = utils.root().get(0);
    let cursor = root.get(2).get(0);
    let content = root.get(0).get(0).get(0);
    assert_eq!(cursor.text(), Some("0:0"));
    assert_eq!(content.text(), Some("Hello Rustaceans\n"));

    // Move cursor
    utils.push_event(TestEvent::Mouse {
        name: MouseEventName::MouseDown,
        cursor: (35.0, 3.0).into(),
        button: Some(MouseButton::Left),
    });

    utils.wait_for_update().await;
    utils.wait_for_update().await;

    // Cursor has been moved
    let root = utils.root().get(0);
    let cursor = root.get(2).get(0);
    assert_eq!(cursor.text(), Some("0:5"));

    // Insert text
    utils.push_event(TestEvent::Keyboard {
        name: KeyboardEventName::KeyDown,
        key: Key::Character("!".to_string()),
        code: Code::Unidentified,
        modifiers: Modifiers::empty(),
    });

    utils.wait_for_update().await;

    // Text and cursor have changed
    let cursor = root.get(2).get(0);
    let content = root.get(0).get(0).get(0);

    assert_eq!(content.text(), Some("Hello! Rustaceans\n"));
    assert_eq!(cursor.text(), Some("0:6"));

    // Second line
    let content = root.get(1).get(0).get(0);
    assert_eq!(content.text(), Some("Hello World"));
}

#[tokio::test]
pub async fn highlight_multiple_lines_single_editor() {
    fn use_editable_app() -> Element {
        let mut editable = use_editable(
            || EditableConfig::new("Hello Rustaceans\n".repeat(2)),
            EditableMode::MultipleLinesSingleEditor,
        );
        let editor = editable.editor().read();
        let cursor_pos = editor.cursor_pos();
        let cursor_reference = editable.cursor_attr();
        let highlights = editable.highlights_attr(0);

        let onmousedown = move |e: MouseEvent| {
            editable.process_event(&EditableEvent::MouseDown(e.data, 0));
        };

        let onmousemove = move |e: MouseEvent| {
            editable.process_event(&EditableEvent::MouseMove(e.data, 0));
        };

        let onglobalkeydown = move |e: Event<KeyboardData>| {
            editable.process_event(&EditableEvent::KeyDown(e.data));
        };

        rsx!(
            rect {
                width: "100%",
                height: "100%",
                background: "white",
                paragraph {
                    cursor_reference,
                    height: "50%",
                    width: "100%",
                    cursor_id: "0",
                    cursor_index: "{cursor_pos}",
                    cursor_color: "black",
                    cursor_mode: "editable",
                    highlights,
                    onglobalkeydown,
                    onmousedown,
                    onmousemove,
                    text {
                        color: "black",
                        "{editor}"
                    }
                }
                label {
                    color: "black",
                    height: "50%",
                    "{editor.cursor_row()}:{editor.cursor_col()}"
                }
            }
        )
    }

    let mut utils = launch_test(use_editable_app);

    let root = utils.root().get(0);

    // Click cursor
    utils.push_event(TestEvent::Mouse {
        name: MouseEventName::MouseDown,
        cursor: (35.0, 3.0).into(),
        button: Some(MouseButton::Left),
    });

    utils.wait_for_update().await;

    // Move cursor
    utils.move_cursor((80., 25.)).await;

    utils.wait_for_update().await;

    let highlights = root.get(0).state().cursor.highlights.clone();
    #[cfg(not(target_os = "macos"))]
    assert_eq!(highlights, Some(vec![(5, 28)]));

    #[cfg(target_os = "macos")]
    assert_eq!(highlights, Some(vec![(5, 27)]));
}

#[tokio::test]
pub async fn highlights_single_line_multiple_editors() {
    fn use_editable_app() -> Element {
        let mut editable = use_editable(
            || EditableConfig::new("Hello Rustaceans\n".repeat(2)),
            EditableMode::SingleLineMultipleEditors,
        );
        let editor = editable.editor().read();

        let onglobalkeydown = move |e: Event<KeyboardData>| {
            editable.process_event(&EditableEvent::KeyDown(e.data));
        };

        rsx!(
            rect {
                width: "100%",
                height: "100%",
                background: "white",
                onglobalkeydown,
                direction: "vertical",
                {editor.lines().enumerate().map(move |(i, line)| {

                    let highlights = editable.highlights_attr(i);

                    let is_line_selected = editable.editor().read().cursor_row() == i;

                    // Only show the cursor in the active line
                    let character_index = if is_line_selected {
                        editable.editor().read().cursor_col().to_string()
                    } else {
                        "none".to_string()
                    };

                    let onmousemove = move |e: MouseEvent| {
                        editable.process_event(&EditableEvent::MouseMove(e.data, i));
                    };

                    let onmousedown = move |e: MouseEvent| {
                        editable.process_event(&EditableEvent::MouseDown(e.data, i));
                    };

                    rsx!(
                        paragraph {
                            cursor_reference: editable.cursor_attr(),
                            width: "100%",
                            height: "30",
                            max_lines: "1",
                            cursor_id: "{i}",
                            cursor_index: "{character_index}",
                            cursor_color: "black",
                            cursor_mode: "editable",
                            onmousemove,
                            onmousedown,
                            highlights,
                            text {
                                color: "black",
                                "{line}"
                            }
                        }
                    )
                })}
                label {
                    color: "black",
                    height: "50%",
                    "{editor.cursor_row()}:{editor.cursor_col()}"
                }
            }
        )
    }

    let mut utils = launch_test(use_editable_app);

    let root = utils.root().get(0);

    // Click cursor
    utils.push_event(TestEvent::Mouse {
        name: MouseEventName::MouseDown,
        cursor: (35.0, 3.0).into(),
        button: Some(MouseButton::Left),
    });

    utils.wait_for_update().await;

    // Move cursor
    utils.move_cursor((35., 3.)).await;

    utils.wait_for_update().await;

    // Move cursor
    utils.move_cursor((80., 35.)).await;

    utils.wait_for_update().await;

    let highlights_1 = root.get(0).state().cursor.highlights.clone();
    assert_eq!(highlights_1, Some(vec![(5, 17)]));

    let highlights_2 = root.get(1).state().cursor.highlights.clone();
    #[cfg(not(target_os = "macos"))]
    assert_eq!(highlights_2, Some(vec![(0, 11)]));

    #[cfg(target_os = "macos")]
    assert_eq!(highlights_2, Some(vec![(0, 10)]));
}

#[tokio::test]
pub async fn special_text_editing() {
    fn special_text_editing_app() -> Element {
        let mut editable = use_editable(
            || EditableConfig::new("你好世界\n👋".to_string()),
            EditableMode::MultipleLinesSingleEditor,
        );
        let cursor_attr = editable.cursor_attr();
        let editor = editable.editor().read();
        let cursor_pos = editor.cursor_pos();

        let onmousedown = move |e: MouseEvent| {
            editable.process_event(&EditableEvent::MouseDown(e.data, 0));
        };

        let onglobalkeydown = move |e: Event<KeyboardData>| {
            editable.process_event(&EditableEvent::KeyDown(e.data));
        };

        rsx!(
            rect {
                width: "100%",
                height: "100%",
                background: "white",
                onmousedown,
                paragraph {
                    cursor_reference: cursor_attr,
                    height: "50%",
                    width: "100%",
                    cursor_id: "0",
                    cursor_index: "{cursor_pos}",
                    cursor_color: "black",
                    cursor_mode: "editable",
                    onglobalkeydown,
                    text {
                        color: "black",
                        "{editor}"
                    }
                }
                label {
                    color: "black",
                    height: "50%",
                    "{editor.cursor_row()}:{editor.cursor_col()}"
                }
            }
        )
    }

    let mut utils = launch_test(special_text_editing_app);

    // Initial state
    let root = utils.root().get(0);
    let cursor = root.get(1).get(0);
    let content = root.get(0).get(0).get(0);
    assert_eq!(cursor.text(), Some("0:0"));
    assert_eq!(content.text(), Some("你好世界\n👋"));

    // Move cursor
    utils.push_event(TestEvent::Mouse {
        name: MouseEventName::MouseDown,
        cursor: (35.0, 3.0).into(),
        button: Some(MouseButton::Left),
    });

    utils.wait_for_update().await;
    utils.wait_for_update().await;

    // Cursor has been moved
    let root = utils.root().get(0);
    let cursor = root.get(1).get(0);
    #[cfg(not(target_os = "linux"))]
    assert_eq!(cursor.text(), Some("0:2"));

    #[cfg(target_os = "linux")]
    assert_eq!(cursor.text(), Some("0:3"));

    // Insert text
    utils.push_event(TestEvent::Keyboard {
        name: KeyboardEventName::KeyDown,
        key: Key::Character("🦀".to_string()),
        code: Code::Unidentified,
        modifiers: Modifiers::empty(),
    });

    utils.wait_for_update().await;

    // Text and cursor have changed
    let cursor = root.get(1).get(0);
    let content = root.get(0).get(0).get(0);
    #[cfg(not(target_os = "linux"))]
    {
        assert_eq!(content.text(), Some("你好🦀世界\n👋"));
        assert_eq!(cursor.text(), Some("0:4"));
    }

    #[cfg(target_os = "linux")]
    {
        assert_eq!(content.text(), Some("你好世🦀界\n👋"));
        assert_eq!(cursor.text(), Some("0:5"));
    }

    // Move cursor to the begining
    utils.push_event(TestEvent::Mouse {
        name: MouseEventName::MouseDown,
        cursor: (3.0, 3.0).into(),
        button: Some(MouseButton::Left),
    });
    utils.wait_for_update().await;
    utils.wait_for_update().await;
    let cursor = root.get(1).get(0);
    assert_eq!(cursor.text(), Some("0:0"));

    // Move cursor with arrow down
    utils.push_event(TestEvent::Keyboard {
        name: KeyboardEventName::KeyDown,
        code: Code::ArrowDown,
        key: Key::ArrowDown,
        modifiers: Modifiers::default(),
    });
    utils.wait_for_update().await;
    let cursor = root.get(1).get(0);
    assert_eq!(cursor.text(), Some("1:0"));

    // Move cursor with arrow right
    utils.push_event(TestEvent::Keyboard {
        name: KeyboardEventName::KeyDown,
        code: Code::ArrowRight,
        key: Key::ArrowRight,
        modifiers: Modifiers::default(),
    });
    utils.wait_for_update().await;
    let cursor = root.get(1).get(0);
    assert_eq!(cursor.text(), Some("1:1"));

    // Move cursor with arrow up
    utils.push_event(TestEvent::Keyboard {
        name: KeyboardEventName::KeyDown,
        code: Code::ArrowUp,
        key: Key::ArrowUp,
        modifiers: Modifiers::default(),
    });
    utils.wait_for_update().await;
    let cursor = root.get(1).get(0);
    assert_eq!(cursor.text(), Some("0:1"));

    // Move cursor with arrow left
    utils.push_event(TestEvent::Keyboard {
        name: KeyboardEventName::KeyDown,
        code: Code::ArrowLeft,
        key: Key::ArrowLeft,
        modifiers: Modifiers::default(),
    });
    utils.wait_for_update().await;
    let cursor = root.get(1).get(0);
    assert_eq!(cursor.text(), Some("0:0"));

    // Move cursor with arrow down, twice
    utils.push_event(TestEvent::Keyboard {
        name: KeyboardEventName::KeyDown,
        code: Code::ArrowDown,
        key: Key::ArrowDown,
        modifiers: Modifiers::default(),
    });
    utils.push_event(TestEvent::Keyboard {
        name: KeyboardEventName::KeyDown,
        code: Code::ArrowDown,
        key: Key::ArrowDown,
        modifiers: Modifiers::default(),
    });
    utils.wait_for_update().await;
    let cursor = root.get(1).get(0);
    // Because there is not a third line, the cursor will be moved to the max right
    assert_eq!(cursor.text(), Some("1:2"));

    // Move cursor with arrow up, twice
    utils.push_event(TestEvent::Keyboard {
        name: KeyboardEventName::KeyDown,
        code: Code::ArrowUp,
        key: Key::ArrowUp,
        modifiers: Modifiers::default(),
    });
    utils.push_event(TestEvent::Keyboard {
        name: KeyboardEventName::KeyDown,
        code: Code::ArrowUp,
        key: Key::ArrowUp,
        modifiers: Modifiers::default(),
    });
    utils.wait_for_update().await;
    let cursor = root.get(1).get(0);
    // Because there is not a line above the first one, the cursor will be moved to the begining
    assert_eq!(cursor.text(), Some("0:0"));
}

#[tokio::test]
pub async fn backspace_remove() {
    fn backspace_remove_app() -> Element {
        let mut editable = use_editable(
            || EditableConfig::new("Hello Rustaceans\nHello Rustaceans".to_string()),
            EditableMode::MultipleLinesSingleEditor,
        );
        let cursor_attr = editable.cursor_attr();
        let editor = editable.editor().read();
        let cursor_pos = editor.cursor_pos();

        let onmousedown = move |e: MouseEvent| {
            editable.process_event(&EditableEvent::MouseDown(e.data, 0));
        };

        let onglobalkeydown = move |e: Event<KeyboardData>| {
            editable.process_event(&EditableEvent::KeyDown(e.data));
        };

        rsx!(
            rect {
                width: "100%",
                height: "100%",
                background: "white",
                onmousedown,
                paragraph {
                    cursor_reference: cursor_attr,
                    height: "50%",
                    width: "100%",
                    cursor_id: "0",
                    cursor_index: "{cursor_pos}",
                    cursor_color: "black",
                    cursor_mode: "editable",
                    onglobalkeydown,
                    text {
                        color: "black",
                        "{editor}"
                    }
                }
                label {
                    color: "black",
                    height: "50%",
                    "{editor.cursor_row()}:{editor.cursor_col()}"
                }
            }
        )
    }

    let mut utils = launch_test(backspace_remove_app);

    // Initial state
    let root = utils.root().get(0);
    let cursor = root.get(1).get(0);
    let content = root.get(0).get(0).get(0);
    assert_eq!(cursor.text(), Some("0:0"));
    assert_eq!(content.text(), Some("Hello Rustaceans\nHello Rustaceans"));

    // Move cursor
    utils.push_event(TestEvent::Mouse {
        name: MouseEventName::MouseDown,
        cursor: (35.0, 3.0).into(),
        button: Some(MouseButton::Left),
    });

    utils.wait_for_update().await;
    utils.wait_for_update().await;

    // Cursor has been moved
    let root = utils.root().get(0);
    let cursor = root.get(1).get(0);
    assert_eq!(cursor.text(), Some("0:5"));

    // Insert text
    utils.push_event(TestEvent::Keyboard {
        name: KeyboardEventName::KeyDown,
        key: Key::Character("🦀".to_string()),
        code: Code::Unidentified,
        modifiers: Modifiers::empty(),
    });

    utils.wait_for_update().await;
    utils.wait_for_update().await;

    // Text and cursor have changed
    let cursor = root.get(1).get(0);
    let content = root.get(0).get(0).get(0);
    assert_eq!(content.text(), Some("Hello🦀 Rustaceans\nHello Rustaceans"));
    assert_eq!(cursor.text(), Some("0:7"));

    // Remove text
    utils.push_event(TestEvent::Keyboard {
        name: KeyboardEventName::KeyDown,
        key: Key::Backspace,
        code: Code::Unidentified,
        modifiers: Modifiers::empty(),
    });

    utils.wait_for_update().await;
    utils.wait_for_update().await;

    // Text and cursor have changed
    let cursor = root.get(1).get(0);
    let content = root.get(0).get(0).get(0);
    assert_eq!(content.text(), Some("Hello Rustaceans\nHello Rustaceans"));
    assert_eq!(cursor.text(), Some("0:5"));
}

#[tokio::test]
pub async fn highlight_shift_click_multiple_lines_single_editor() {
    fn use_editable_app() -> Element {
        let mut editable = use_editable(
            || EditableConfig::new("Hello Rustaceans\n".repeat(2)),
            EditableMode::MultipleLinesSingleEditor,
        );
        let editor = editable.editor().read();
        let cursor_pos = editor.cursor_pos();
        let cursor_reference = editable.cursor_attr();
        let highlights = editable.highlights_attr(0);

        let onmousedown = move |e: MouseEvent| {
            editable.process_event(&EditableEvent::MouseDown(e.data, 0));
        };

        let onmousemove = move |e: MouseEvent| {
            editable.process_event(&EditableEvent::MouseMove(e.data, 0));
        };

        let onglobalkeydown = move |e: Event<KeyboardData>| {
            editable.process_event(&EditableEvent::KeyDown(e.data));
        };

        let onclick = move |_: MouseEvent| {
            editable.process_event(&EditableEvent::Click);
        };

        rsx!(
            rect {
                width: "100%",
                height: "100%",
                background: "white",
                paragraph {
                    cursor_reference,
                    height: "50%",
                    width: "100%",
                    cursor_id: "0",
                    cursor_index: "{cursor_pos}",
                    cursor_color: "black",
                    cursor_mode: "editable",
                    highlights,
                    onglobalkeydown,
                    onclick,
                    onmousedown,
                    onmousemove,
                    text {
                        color: "black",
                        "{editor}"
                    }
                }
                label {
                    color: "black",
                    height: "50%",
                    "{editor.cursor_row()}:{editor.cursor_col()}"
                }
            }
        )
    }

    let mut utils = launch_test(use_editable_app);

    let root = utils.root().get(0);

    // Click cursor
    utils.click_cursor((35., 3.)).await;

    // Press shift
    utils.push_event(TestEvent::Keyboard {
        name: KeyboardEventName::KeyDown,
        key: Key::Shift,
        code: Code::ShiftLeft,
        modifiers: Modifiers::default(),
    });
    utils.wait_for_update().await;

    // Move and click cursor
    utils.click_cursor((80., 25.)).await;

    utils.wait_for_update().await;

    let highlights = root.get(0).state().cursor.highlights.clone();
    #[cfg(not(target_os = "macos"))]
    assert_eq!(highlights, Some(vec![(5, 28)]));

    #[cfg(target_os = "macos")]
    assert_eq!(highlights, Some(vec![(5, 27)]));
}

#[tokio::test]
pub async fn highlights_shift_click_single_line_multiple_editors() {
    fn use_editable_app() -> Element {
        let mut editable = use_editable(
            || EditableConfig::new("Hello Rustaceans\n".repeat(2)),
            EditableMode::SingleLineMultipleEditors,
        );
        let editor = editable.editor().read();

        let onglobalkeydown = move |e: Event<KeyboardData>| {
            editable.process_event(&EditableEvent::KeyDown(e.data));
        };

        rsx!(
            rect {
                width: "100%",
                height: "100%",
                background: "white",
                onglobalkeydown,
                direction: "vertical",
                {editor.lines().enumerate().map(move |(i, line)| {

                    let highlights = editable.highlights_attr(i);

                    let is_line_selected = editable.editor().read().cursor_row() == i;

                    // Only show the cursor in the active line
                    let character_index = if is_line_selected {
                        editable.editor().read().cursor_col().to_string()
                    } else {
                        "none".to_string()
                    };

                    let onmousemove = move |e: MouseEvent| {
                        editable.process_event(&EditableEvent::MouseMove(e.data, i));
                    };

                    let onmousedown = move |e: MouseEvent| {
                        editable.process_event(&EditableEvent::MouseDown(e.data, i));
                    };

                    let onclick = move |_: MouseEvent| {
                        editable.process_event(&EditableEvent::Click);
                    };

                    rsx!(
                        paragraph {
                            cursor_reference: editable.cursor_attr(),
                            width: "100%",
                            height: "30",
                            max_lines: "1",
                            cursor_id: "{i}",
                            cursor_index: "{character_index}",
                            cursor_color: "black",
                            cursor_mode: "editable",
                            onclick,
                            onmousemove,
                            onmousedown,
                            highlights,
                            text {
                                color: "black",
                                "{line}"
                            }
                        }
                    )
                })}
                label {
                    color: "black",
                    height: "50%",
                    "{editor.cursor_row()}:{editor.cursor_col()}"
                }
            }
        )
    }

    let mut utils = launch_test(use_editable_app);

    let root = utils.root().get(0);

    // Click cursor
    utils.click_cursor((35., 3.)).await;

    // Press shift
    utils.push_event(TestEvent::Keyboard {
        name: KeyboardEventName::KeyDown,
        key: Key::Shift,
        code: Code::ShiftLeft,
        modifiers: Modifiers::default(),
    });
    utils.wait_for_update().await;

    // Click cursor
    utils.click_cursor((80., 35.)).await;

    utils.wait_for_update().await;

    let highlights_1 = root.get(0).state().cursor.highlights.clone();

    assert_eq!(highlights_1, Some(vec![(5, 17)]));

    let highlights_2 = root.get(1).state().cursor.highlights.clone();

    #[cfg(not(target_os = "macos"))]
    assert_eq!(highlights_2, Some(vec![(0, 11)]));

    #[cfg(target_os = "macos")]
    assert_eq!(highlights_2, Some(vec![(0, 10)]));
}

#[tokio::test]
pub async fn highlight_all_text() {
    fn use_editable_app() -> Element {
        let mut editable = use_editable(
            || EditableConfig::new("Hello Rustaceans\n".repeat(2)),
            EditableMode::MultipleLinesSingleEditor,
        );
        let editor = editable.editor().read();
        let cursor_pos = editor.cursor_pos();
        let cursor_reference = editable.cursor_attr();
        let highlights = editable.highlights_attr(0);

        let onmousedown = move |e: MouseEvent| {
            editable.process_event(&EditableEvent::MouseDown(e.data, 0));
        };

        let onmousemove = move |e: MouseEvent| {
            editable.process_event(&EditableEvent::MouseMove(e.data, 0));
        };

        let onglobalkeydown = move |e: Event<KeyboardData>| {
            editable.process_event(&EditableEvent::KeyDown(e.data));
        };

        let onclick = move |_: MouseEvent| {
            editable.process_event(&EditableEvent::Click);
        };

        rsx!(
            rect {
                width: "100%",
                height: "100%",
                background: "white",
                paragraph {
                    cursor_reference,
                    height: "50%",
                    width: "100%",
                    cursor_id: "0",
                    cursor_index: "{cursor_pos}",
                    cursor_color: "black",
                    cursor_mode: "editable",
                    highlights,
                    onglobalkeydown,
                    onclick,
                    onmousedown,
                    onmousemove,
                    text {
                        color: "black",
                        "{editor}"
                    }
                }
                label {
                    color: "black",
                    height: "50%",
                    "{editor.cursor_row()}:{editor.cursor_col()}"
                }
            }
        )
    }

    let mut utils = launch_test(use_editable_app);

    let root = utils.root().get(0);

    #[cfg(target_os = "macos")]
    let modifiers = Modifiers::META;

    #[cfg(not(target_os = "macos"))]
    let modifiers = Modifiers::CONTROL;

    // Select all text
    utils.push_event(TestEvent::Keyboard {
        name: KeyboardEventName::KeyDown,
        key: Key::Character("a".to_string()),
        code: Code::KeyA,
        modifiers,
    });
    utils.wait_for_update().await;
    utils.wait_for_update().await;

    let highlights = root.get(0).state().cursor.highlights.clone();

    let start = 0;
    let end = 34;

    assert_eq!(highlights, Some(vec![(start, end)]))
}

#[tokio::test]
pub async fn replace_text() {
    fn replace_text_app() -> Element {
        let mut editable = use_editable(
            || EditableConfig::new("Hello Rustaceans\nHello Rustaceans".to_string()),
            EditableMode::MultipleLinesSingleEditor,
        );
        let cursor_attr = editable.cursor_attr();
        let editor = editable.editor().read();
        let cursor_pos = editor.cursor_pos();
        let highlights = editable.highlights_attr(0);

        let onmousedown = move |e: MouseEvent| {
            editable.process_event(&EditableEvent::MouseDown(e.data, 0));
        };

        let onglobalkeydown = move |e: Event<KeyboardData>| {
            editable.process_event(&EditableEvent::KeyDown(e.data));
        };

        let onclick = move |_: MouseEvent| {
            editable.process_event(&EditableEvent::Click);
        };

        rsx!(
            rect {
                width: "100%",
                height: "100%",
                background: "white",
                onmousedown,
                onclick,
                paragraph {
                    cursor_reference: cursor_attr,
                    height: "50%",
                    width: "100%",
                    cursor_id: "0",
                    cursor_index: "{cursor_pos}",
                    cursor_color: "black",
                    cursor_mode: "editable",
                    onglobalkeydown,
                    highlights,
                    text {
                        color: "black",
                        "{editor}"
                    }
                }
                label {
                    color: "black",
                    height: "50%",
                    "{editor.cursor_row()}:{editor.cursor_col()}"
                }
            }
        )
    }

    let mut utils = launch_test(replace_text_app);

    // Initial state
    let root = utils.root().get(0);
    let cursor = root.get(1).get(0);
    let content = root.get(0).get(0).get(0);
    assert_eq!(cursor.text(), Some("0:0"));
    assert_eq!(content.text(), Some("Hello Rustaceans\nHello Rustaceans"));

    // Move cursor
    //  utils.push_event(MouseEvent::builder().name(PlatformEventName::));
    utils.push_event(TestEvent::Mouse {
        name: MouseEventName::MouseDown,
        cursor: (35.0, 3.0).into(),
        button: Some(MouseButton::Left),
    });

    utils.wait_for_update().await;
    utils.wait_for_update().await;

    // Cursor has been moved
    let root = utils.root().get(0);
    let cursor = root.get(1).get(0);
    assert_eq!(cursor.text(), Some("0:5"));

    // Click cursor
    utils.click_cursor((35., 3.)).await;

    // Press shift
    utils.push_event(TestEvent::Keyboard {
        name: KeyboardEventName::KeyDown,
        key: Key::Shift,
        code: Code::ShiftLeft,
        modifiers: Modifiers::default(),
    });
    utils.wait_for_update().await;

    // Move and click cursor
    utils.click_cursor((80., 3.)).await;

    // Insert text
    utils.push_event(TestEvent::Keyboard {
        name: KeyboardEventName::KeyDown,
        key: Key::Character("🦀".to_string()),
        code: Code::Unidentified,
        modifiers: Modifiers::empty(),
    });

    utils.wait_for_update().await;
    utils.wait_for_update().await;

    // Text and cursor have changed
    let cursor = root.get(1).get(0);
    let content = root.get(0).get(0).get(0);
    #[cfg(not(target_os = "macos"))]
    {
        assert_eq!(content.text(), Some("Hello🦀ceans\nHello Rustaceans"));
        assert_eq!(cursor.text(), Some("0:7"));
    }

    #[cfg(target_os = "macos")]
    {
        assert_eq!(content.text(), Some("Hello🦀aceans\nHello Rustaceans"));
        assert_eq!(cursor.text(), Some("0:7"));
    }
}

#[tokio::test]
pub async fn navigate_empty_lines() {
    fn replace_text_app() -> Element {
        let mut editable = use_editable(
            || EditableConfig::new("".to_string()),
            EditableMode::MultipleLinesSingleEditor,
        );
        let cursor_attr = editable.cursor_attr();
        let editor = editable.editor().read();
        let cursor_pos = editor.cursor_pos();
        let highlights = editable.highlights_attr(0);

        let onmousedown = move |e: MouseEvent| {
            editable.process_event(&EditableEvent::MouseDown(e.data, 0));
        };

        let onglobalkeydown = move |e: Event<KeyboardData>| {
            editable.process_event(&EditableEvent::KeyDown(e.data));
        };

        let onclick = move |_: MouseEvent| {
            editable.process_event(&EditableEvent::Click);
        };

        rsx!(
            rect {
                width: "100%",
                height: "100%",
                background: "white",
                onmousedown,
                onclick,
                paragraph {
                    cursor_reference: cursor_attr,
                    height: "50%",
                    width: "100%",
                    cursor_id: "0",
                    cursor_index: "{cursor_pos}",
                    cursor_color: "black",
                    cursor_mode: "editable",
                    onglobalkeydown,
                    highlights,
                    text {
                        color: "black",
                        "{editor}"
                    }
                }
                label {
                    color: "black",
                    height: "50%",
                    "{editor.cursor_row()}:{editor.cursor_col()}"
                }
            }
        )
    }

    let mut utils = launch_test(replace_text_app);

    // Initial state
    let root = utils.root().get(0);
    let cursor = root.get(1).get(0);
    let content = root.get(0).get(0).get(0);
    assert_eq!(cursor.text(), Some("0:0"));
    assert_eq!(content.text(), Some(""));

    // Press Enter
    utils.push_event(TestEvent::Keyboard {
        name: KeyboardEventName::KeyDown,
        key: Key::Enter,
        code: Code::Enter,
        modifiers: Modifiers::default(),
    });
    utils.wait_for_update().await;

    // Content has been edited
    let content = root.get(0).get(0).get(0);
    assert_eq!(content.text(), Some("\n"));

    // Cursor has been moved
    let root = utils.root().get(0);
    let cursor = root.get(1).get(0);
    assert_eq!(cursor.text(), Some("1:0"));

    // Press ArrowUp
    utils.push_event(TestEvent::Keyboard {
        name: KeyboardEventName::KeyDown,
        key: Key::ArrowUp,
        code: Code::ArrowUp,
        modifiers: Modifiers::default(),
    });
    utils.wait_for_update().await;

    // Cursor has been moved
    let cursor = root.get(1).get(0);
    assert_eq!(cursor.text(), Some("0:0"));

    // Press ArrowDown
    utils.push_event(TestEvent::Keyboard {
        name: KeyboardEventName::KeyDown,
        key: Key::ArrowDown,
        code: Code::ArrowDown,
        modifiers: Modifiers::default(),
    });
    utils.wait_for_update().await;

    // Cursor has been moved
    let cursor = root.get(1).get(0);
    assert_eq!(cursor.text(), Some("1:0"));
}
