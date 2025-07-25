use dioxus_core::{
    prelude::{
        consume_context,
        provide_context,
        spawn,
    },
    use_hook,
};
use dioxus_signals::{
    Readable,
    Signal,
    Writable,
};
use freya_core::types::NativePlatformReceiver;

use crate::use_init_asset_cacher;

/// Keep some native features (focused element, preferred theme, etc) on sync between the platform and the components
pub fn use_init_native_platform() {
    // Init the global asset cacher
    use_init_asset_cacher();

    // Init the signals with platform values
    use_hook(|| {
        let mut platform_receiver = consume_context::<NativePlatformReceiver>();
        let platform_state = platform_receiver.borrow();

        let mut preferred_theme = Signal::new(platform_state.preferred_theme);
        let mut focused_id = Signal::new(platform_state.focused_accessibility_id);
        let mut focused_node = Signal::new(platform_state.focused_accessibility_node.clone());
        let mut navigation_mode = Signal::new(platform_state.navigation_mode);
        let mut information = Signal::new(platform_state.information);

        drop(platform_state);

        // Listen for any changes during the execution of the app
        spawn(async move {
            while platform_receiver.changed().await.is_ok() {
                let state = platform_receiver.borrow();
                if *focused_id.peek() != state.focused_accessibility_id {
                    *focused_id.write() = state.focused_accessibility_id;
                }

                if *focused_node.peek() != state.focused_accessibility_node {
                    *focused_node.write() = state.focused_accessibility_node.clone();
                }

                if *preferred_theme.peek() != state.preferred_theme {
                    *preferred_theme.write() = state.preferred_theme;
                }

                if *navigation_mode.peek() != state.navigation_mode {
                    *navigation_mode.write() = state.navigation_mode;
                }

                if *information.peek() != state.information {
                    *information.write() = state.information;
                }
            }
        });

        provide_context(preferred_theme);
        provide_context(navigation_mode);
        provide_context(information);
        provide_context(focused_id);
        provide_context(focused_node);
    });
}

#[cfg(test)]
mod test {
    use freya::prelude::*;
    use freya_core::accessibility::ACCESSIBILITY_ROOT_ID;
    use freya_testing::prelude::*;

    #[tokio::test]
    pub async fn focus_accessibility() {
        #[allow(non_snake_case)]
        fn OtherChild() -> Element {
            let mut focus_manager = use_focus();

            rsx!(rect {
                a11y_id: focus_manager.attribute(),
                width: "100%",
                height: "50%",
                onclick: move |_| focus_manager.request_focus(),
            })
        }

        fn use_focus_app() -> Element {
            rsx!(
                rect {
                    width: "100%",
                    height: "100%",
                    OtherChild {}
                    OtherChild {}
                }
            )
        }

        let mut utils = launch_test_with_config(
            use_focus_app,
            TestingConfig::<()> {
                size: (100.0, 100.0).into(),
                ..TestingConfig::default()
            },
        );

        // Initial state
        utils.wait_for_update().await;
        assert_eq!(utils.focus_id(), ACCESSIBILITY_ROOT_ID);

        // Click on the first rect
        utils.click_cursor((5., 5.)).await;

        // First rect is now focused
        utils.wait_for_update().await;
        utils.wait_for_update().await;
        let first_focus_id = utils.focus_id();
        assert_ne!(first_focus_id, ACCESSIBILITY_ROOT_ID);

        // Click on the second rect
        utils.click_cursor((5., 75.)).await;

        // Second rect is now focused
        utils.wait_for_update().await;
        utils.wait_for_update().await;
        let second_focus_id = utils.focus_id();
        assert_ne!(first_focus_id, second_focus_id);
        assert_ne!(second_focus_id, ACCESSIBILITY_ROOT_ID);
    }

    #[tokio::test]
    pub async fn uncontrolled_focus_accessibility() {
        #[allow(non_snake_case)]
        fn OtherChild() -> Element {
            let focus = use_focus();
            rsx!(rect {
                a11y_id: focus.attribute(),
                width: "100%",
                height: "50%",
            })
        }

        fn use_focus_app() -> Element {
            rsx!(
                rect {
                    width: "100%",
                    height: "100%",
                    OtherChild {},
                    OtherChild {}
                }
            )
        }

        let mut utils = launch_test_with_config(
            use_focus_app,
            TestingConfig::<()> {
                size: (100.0, 100.0).into(),
                ..TestingConfig::default()
            },
        );

        // Initial state
        utils.wait_for_update().await;
        assert_eq!(utils.focus_id(), ACCESSIBILITY_ROOT_ID);

        // Navigate to the first rect
        utils.push_event(TestEvent::Keyboard {
            name: KeyboardEventName::KeyDown,
            key: Key::Tab,
            code: Code::Tab,
            modifiers: Modifiers::default(),
        });
        utils.wait_for_update().await;

        // First rect is now focused
        utils.wait_for_update().await;
        utils.wait_for_update().await;
        let first_focus_id = utils.focus_id();
        assert_ne!(first_focus_id, ACCESSIBILITY_ROOT_ID);

        // Navigate to the second rect
        utils.push_event(TestEvent::Keyboard {
            name: KeyboardEventName::KeyDown,
            key: Key::Tab,
            code: Code::Tab,
            modifiers: Modifiers::default(),
        });
        utils.wait_for_update().await;

        utils.wait_for_update().await;
        utils.wait_for_update().await;
        let second_focus_id = utils.focus_id();
        assert_ne!(first_focus_id, second_focus_id);
        assert_ne!(second_focus_id, ACCESSIBILITY_ROOT_ID);
    }

    #[tokio::test]
    pub async fn auto_focus_accessibility() {
        fn use_focus_app() -> Element {
            let focus_1 = use_focus();
            let focus_2 = use_focus();
            rsx!(
                rect {
                    a11y_id: focus_1.attribute(),
                    a11y_auto_focus: "true",
                }
                rect {
                    a11y_id: focus_2.attribute(),
                    a11y_auto_focus: "true",
                }
            )
        }

        let mut utils = launch_test_with_config(
            use_focus_app,
            TestingConfig::<()> {
                size: (100.0, 100.0).into(),
                ..TestingConfig::default()
            },
        );

        utils.wait_for_update().await;
        assert_ne!(utils.focus_id(), ACCESSIBILITY_ROOT_ID); // Will focus the second rect
    }
}
