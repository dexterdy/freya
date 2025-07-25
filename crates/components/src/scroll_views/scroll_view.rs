use dioxus::prelude::*;
use freya_elements::{
    self as dioxus_elements,
    events::{
        keyboard::Key,
        KeyboardEvent,
        MouseEvent,
        WheelEvent,
    },
    WheelSource,
};
use freya_hooks::{
    use_applied_theme,
    use_focus,
    use_node_from_signal,
    ScrollBarThemeWith,
};

use super::use_scroll_controller::ScrollController;
use crate::{
    get_container_sizes,
    get_corrected_scroll_position,
    get_scroll_position_from_cursor,
    get_scroll_position_from_wheel,
    get_scrollbar_pos_and_size,
    is_scrollbar_visible,
    manage_key_event,
    scroll_views::use_scroll_controller::{
        use_scroll_controller,
        ScrollConfig,
    },
    Axis,
    ScrollBar,
    ScrollThumb,
    SCROLL_SPEED_MULTIPLIER,
};

/// Properties for the [`ScrollView`] component.
#[derive(Props, Clone, PartialEq)]
pub struct ScrollViewProps {
    /// Width of the ScrollView container. Default to `fill`.
    #[props(default = "fill".into())]
    pub width: String,
    /// Height of the ScrollView container. Default to `fill`.
    #[props(default = "fill".into())]
    pub height: String,
    /// Minimum width of the ScrollView container.
    pub min_width: Option<f32>,
    /// Minimum height of the ScrollView container.
    pub min_height: Option<f32>,
    /// Maximum width of the ScrollView container.
    pub max_width: Option<f32>,
    /// Maximum height of the ScrollView container.
    pub max_height: Option<f32>,
    /// Padding of the ScrollView container.
    #[props(default = "0".to_string())]
    pub padding: String,
    /// Spacing for the ScrollView container.
    #[props(default = "0".to_string())]
    pub spacing: String,
    /// Theme override for the scrollbars.
    pub scrollbar_theme: Option<ScrollBarThemeWith>,
    /// Inner children for the ScrollView.
    pub children: Element,
    /// Direction of the ScrollView, `vertical` or `horizontal`.
    #[props(default = "vertical".to_string(), into)]
    pub direction: String,
    /// Show the scrollbar, visible by default.
    #[props(default = true, into)]
    pub show_scrollbar: bool,
    /// Enable scrolling with arrow keys.
    #[props(default = true, into)]
    pub scroll_with_arrows: bool,
    /// Custom Scroll Controller for the ScrollView.
    pub scroll_controller: Option<ScrollController>,
    /// If `false` (default), wheel scroll with no shift will scroll vertically no matter the direction.
    /// If `true`, wheel scroll with no shift will scroll horizontally.
    #[props(default = false)]
    pub invert_scroll_wheel: bool,
}

/// Scrollable area with bidirectional support and scrollbars.
///
/// # Example
///
/// ```no_run
/// # use freya::prelude::*;
/// fn app() -> Element {
///     rsx!(
///         ScrollView {
///             rect {
///                 background: "blue",
///                 height: "400",
///                 width: "100%"
///             }
///             rect {
///                 background: "red",
///                 height: "400",
///                 width: "100%"
///              }
///         }
///     )
/// }
/// ```
///
/// # With a Scroll Controller
///
/// ```rust
/// # use freya::prelude::*;
/// fn app() -> Element {
///     let mut scroll_controller = use_scroll_controller(|| ScrollConfig::default());
///
///     rsx!(
///         ScrollView {
///             scroll_controller,
///             rect {
///                 background: "blue",
///                 height: "400",
///                 width: "100%"
///             }
///             Button {
///                 onpress: move |_| {
///                    scroll_controller.scroll_to(ScrollPosition::Start, ScrollDirection::Vertical);
///                 },
///                 label {
///                     label {
///                         "Scroll up"
///                     }
///                 }
///             }
///             rect {
///                 background: "red",
///                 height: "400",
///                 width: "100%"
///             }
///         }
///     )
/// }
/// # use freya_testing::prelude::*;
/// # launch_doc(|| {
/// #   rsx!(
/// #       Preview {
/// #           ScrollView {
/// #               label {
/// #                   "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Vestibulum laoreet tristique diam, ut gravida enim. Phasellus viverra vitae risus sit amet iaculis. Morbi porttitor quis nisl eu vulputate. Etiam vitae ligula a purus suscipit iaculis non ac risus. Suspendisse potenti. Aenean orci massa, ornare ut elit id, tristique commodo dui. Vestibulum laoreet tristique diam, ut gravida enim. Phasellus viverra vitae risus sit amet iaculis. Vestibulum laoreet tristique diam, ut gravida enim. Phasellus viverra vitae risus sit amet iaculis. Vestibulum laoreet tristique diam, ut gravida enim. Phasellus viverra vitae risus sit amet iaculis."
/// #               }
/// #           }
/// #       }
/// #   )
/// # }, (250., 250.).into(), "./images/gallery_scroll_view.png");
/// ```
///
/// # Preview
/// ![ScrollView Preview][scroll_view]
#[cfg_attr(feature = "docs",
    doc = embed_doc_image::embed_image!("scroll_view", "images/gallery_scroll_view.png")
)]
#[allow(non_snake_case)]
pub fn ScrollView(
    ScrollViewProps {
        width,
        height,
        min_width,
        min_height,
        max_width,
        max_height,
        padding,
        spacing,
        scrollbar_theme,
        children,
        direction,
        show_scrollbar,
        scroll_with_arrows,
        scroll_controller,
        invert_scroll_wheel,
    }: ScrollViewProps,
) -> Element {
    let mut clicking_scrollbar = use_signal::<Option<(Axis, f64)>>(|| None);
    let mut clicking_shift = use_signal(|| false);
    let mut clicking_alt = use_signal(|| false);
    let mut scroll_controller =
        scroll_controller.unwrap_or_else(|| use_scroll_controller(ScrollConfig::default));
    let (mut scrolled_x, mut scrolled_y) = scroll_controller.into();
    let (node_ref, size) = use_node_from_signal(|| scroll_controller.layout());

    let mut focus = use_focus();
    let applied_scrollbar_theme = use_applied_theme!(&scrollbar_theme, scroll_bar);

    scroll_controller.use_apply(size.inner.width, size.inner.height);

    let vertical_scrollbar_is_visible = is_scrollbar_visible(
        show_scrollbar,
        size.inner.height.floor(),
        size.area.height().floor(),
    );
    let horizontal_scrollbar_is_visible = is_scrollbar_visible(
        show_scrollbar,
        size.inner.width.floor(),
        size.area.width().floor(),
    );

    let (container_width, content_width) = get_container_sizes(&width);
    let (container_height, content_height) = get_container_sizes(&height);

    let corrected_scrolled_y = get_corrected_scroll_position(
        size.inner.height,
        size.area.height(),
        *scrolled_y.read() as f32,
    );
    let corrected_scrolled_x = get_corrected_scroll_position(
        size.inner.width,
        size.area.width(),
        *scrolled_x.read() as f32,
    );

    let (scrollbar_y, scrollbar_height) =
        get_scrollbar_pos_and_size(size.inner.height, size.area.height(), corrected_scrolled_y);
    let (scrollbar_x, scrollbar_width) =
        get_scrollbar_pos_and_size(size.inner.width, size.area.width(), corrected_scrolled_x);

    // Moves the axis when the user scrolls in the container
    let onwheel = move |e: WheelEvent| {
        let speed_multiplier = if *clicking_alt.peek() {
            SCROLL_SPEED_MULTIPLIER
        } else {
            1.0
        };

        // Only invert direction on deviced-sourced wheel events
        let invert_direction = e.data.get_source() == WheelSource::Device
            && (clicking_shift() || invert_scroll_wheel)
            && (!clicking_shift() || !invert_scroll_wheel);

        let (x_movement, y_movement) = if invert_direction {
            (
                e.get_delta_y() as f32 * speed_multiplier,
                e.get_delta_x() as f32 * speed_multiplier,
            )
        } else {
            (
                e.get_delta_x() as f32 * speed_multiplier,
                e.get_delta_y() as f32 * speed_multiplier,
            )
        };

        let scroll_position_y = get_scroll_position_from_wheel(
            y_movement,
            size.inner.height,
            size.area.height(),
            corrected_scrolled_y,
        );

        // Only scroll when there is still area to scroll
        if *scrolled_y.peek() != scroll_position_y {
            e.stop_propagation();
            *scrolled_y.write() = scroll_position_y;
        }

        let scroll_position_x = get_scroll_position_from_wheel(
            x_movement,
            size.inner.width,
            size.area.width(),
            corrected_scrolled_x,
        );

        // Only scroll when there is still area to scroll
        if *scrolled_x.peek() != scroll_position_x {
            e.stop_propagation();
            *scrolled_x.write() = scroll_position_x;
        }
    };

    // Drag the scrollbars
    let oncaptureglobalmousemove = move |e: MouseEvent| {
        let clicking_scrollbar = clicking_scrollbar.peek();

        if let Some((Axis::Y, y)) = *clicking_scrollbar {
            let coordinates = e.get_element_coordinates();
            let cursor_y = coordinates.y - y - size.area.min_y() as f64;

            let scroll_position = get_scroll_position_from_cursor(
                cursor_y as f32,
                size.inner.height,
                size.area.height(),
            );

            *scrolled_y.write() = scroll_position;
        } else if let Some((Axis::X, x)) = *clicking_scrollbar {
            let coordinates = e.get_element_coordinates();
            let cursor_x = coordinates.x - x - size.area.min_x() as f64;

            let scroll_position = get_scroll_position_from_cursor(
                cursor_x as f32,
                size.inner.width,
                size.area.width(),
            );

            *scrolled_x.write() = scroll_position;
        }

        if clicking_scrollbar.is_some() {
            e.prevent_default();
            focus.request_focus();
        }
    };

    let onglobalkeydown = move |e: KeyboardEvent| {
        match &e.key {
            Key::Shift => {
                clicking_shift.set(true);
            }
            Key::Alt => {
                clicking_alt.set(true);
            }
            k => {
                if !focus.is_focused() {
                    return;
                }
                if !scroll_with_arrows
                    && (k == &Key::ArrowUp
                        || k == &Key::ArrowRight
                        || k == &Key::ArrowDown
                        || k == &Key::ArrowLeft)
                {
                    return;
                }

                let x = corrected_scrolled_x;
                let y = corrected_scrolled_y;
                let inner_height = size.inner.height;
                let inner_width = size.inner.width;
                let viewport_height = size.area.height();
                let viewport_width = size.area.width();

                let (x, y) = manage_key_event(
                    e,
                    (x, y),
                    inner_height,
                    inner_width,
                    viewport_height,
                    viewport_width,
                );

                scrolled_x.set(x as i32);
                scrolled_y.set(y as i32);
            }
        };
    };

    let onglobalkeyup = move |e: KeyboardEvent| {
        if e.key == Key::Shift {
            clicking_shift.set(false);
        } else if e.key == Key::Alt {
            clicking_alt.set(false);
        }
    };

    // Mark the Y axis scrollbar as the one being dragged
    let onmousedown_y = move |e: MouseEvent| {
        let coordinates = e.get_element_coordinates();
        *clicking_scrollbar.write() = Some((Axis::Y, coordinates.y));
    };

    // Mark the X axis scrollbar as the one being dragged
    let onmousedown_x = move |e: MouseEvent| {
        let coordinates = e.get_element_coordinates();
        *clicking_scrollbar.write() = Some((Axis::X, coordinates.x));
    };

    // Unmark any scrollbar
    let onglobalpointerup = move |_| {
        if clicking_scrollbar.peek().is_some() {
            *clicking_scrollbar.write() = None;
        }
    };

    let is_scrolling_x = clicking_scrollbar
        .read()
        .as_ref()
        .map(|f| f.0 == Axis::X)
        .unwrap_or_default();
    let is_scrolling_y = clicking_scrollbar
        .read()
        .as_ref()
        .map(|f| f.0 == Axis::Y)
        .unwrap_or_default();

    let a11y_id = focus.attribute();

    rsx!(
        rect {
            a11y_role: "scroll-view",
            a11y_scroll_y: "{corrected_scrolled_y}",
            a11y_scroll_x: "{corrected_scrolled_x}",
            overflow: "clip",
            direction: "horizontal",
            width: width.clone(),
            height: height.clone(),
            min_width: min_width.map(|x| x.to_string()),
            min_height: min_height.map(|x| x.to_string()),
            max_width: max_width.map(|x| x.to_string()),
            max_height: max_height.map(|x| x.to_string()),
            onglobalpointerup,
            oncaptureglobalmousemove,
            onglobalkeydown,
            onglobalkeyup,
            onwheel,
            a11y_id,
            a11y_focusable: "false",
            rect {
                direction: "vertical",
                width: container_width,
                height: container_height,
                rect {
                    overflow: "clip",
                    spacing,
                    padding,
                    width: content_width,
                    height: content_height,
                    min_width: min_width.map(|x| x.to_string()),
                    min_height: min_height.map(|x| x.to_string()),
                    max_width: max_width.map(|x| x.to_string()),
                    max_height: max_height.map(|x| x.to_string()),
                    direction: direction,
                    offset_y: "{corrected_scrolled_y}",
                    offset_x: "{corrected_scrolled_x}",
                    reference: node_ref,
                    {children}
                }
                if show_scrollbar && horizontal_scrollbar_is_visible {
                    ScrollBar {
                        size: &applied_scrollbar_theme.size,
                        offset_x: scrollbar_x,
                        clicking_scrollbar: is_scrolling_x,
                        theme: scrollbar_theme.clone(),
                        ScrollThumb {
                            clicking_scrollbar: is_scrolling_x,
                            onmousedown: onmousedown_x,
                            width: "{scrollbar_width}",
                            height: "100%",
                            theme: scrollbar_theme.clone()
                        }
                    }
                }
            }
            if show_scrollbar && vertical_scrollbar_is_visible {
                ScrollBar {
                    is_vertical: true,
                    size: &applied_scrollbar_theme.size,
                    offset_y: scrollbar_y,
                    clicking_scrollbar: is_scrolling_y,
                    theme: scrollbar_theme.clone(),
                    ScrollThumb {
                        clicking_scrollbar: is_scrolling_y,
                        onmousedown: onmousedown_y,
                        width: "100%",
                        height: "{scrollbar_height}",
                        theme: scrollbar_theme
                    }
                }
            }
        }
    )
}

#[cfg(test)]
mod test {
    use freya::prelude::*;
    use freya_testing::prelude::*;

    #[tokio::test]
    pub async fn scroll_view_wheel() {
        fn scroll_view_wheel_app() -> Element {
            rsx!(
                ScrollView {
                    rect {
                        height: "200",
                        width: "200",
                    }
                    rect {
                        height: "200",
                        width: "200",
                    }
                    rect {
                        height: "200",
                        width: "200",
                    }
                    rect {
                        height: "200",
                        width: "200",
                    }
                }
            )
        }

        let mut utils = launch_test(scroll_view_wheel_app);
        let root = utils.root();
        let content = root.get(0).get(0).get(0);
        utils.wait_for_update().await;

        // Only the first three items are visible
        // Scrollview height is 500 and the user hasn't scrolled yet
        assert!(content.get(0).is_visible()); // 1. 0   -> 200, 200 < 500
        assert!(content.get(1).is_visible()); // 2. 200 -> 400, 200 < 500
        assert!(content.get(2).is_visible()); // 3. 400 -> 600, 400 < 500
        assert!(!content.get(3).is_visible()); // 4. 600 -> 800, 600 is NOT < 500, which means it is not visible.

        utils.push_event(TestEvent::Wheel {
            name: WheelEventName::Wheel,
            scroll: (0., -300.).into(),
            cursor: (5., 5.).into(),
        });

        utils.wait_for_update().await;

        // Only the last three items are visible
        // Scrollview height is 500 but the user has scrolled 300 pixels
        assert!(!content.get(0).is_visible()); // 1. 0   -> 200, 200 is NOT > 300, which means it is not visible.
        assert!(content.get(1).is_visible()); // 2. 200 -> 400, 400 > 300
        assert!(content.get(2).is_visible()); // 3. 400 -> 600, 600 > 300
        assert!(content.get(3).is_visible()); // 4. 600 -> 800, 800 > 300
    }

    #[tokio::test]
    pub async fn scroll_view_scrollbar() {
        fn scroll_view_scrollbar_app() -> Element {
            rsx!(
                ScrollView {
                    rect {
                        height: "200",
                        width: "200",
                    }
                    rect {
                        height: "200",
                        width: "200",
                    }
                    rect {
                        height: "200",
                        width: "200",
                    }
                    rect {
                        height: "200",
                        width: "200",
                    }
                }
            )
        }

        let mut utils = launch_test(scroll_view_scrollbar_app);
        let root = utils.root();
        let content = root.get(0).get(0).get(0);
        utils.wait_for_update().await;

        // Only the first three items are visible
        // Scrollview height is 500 and the user hasn't scrolled yet
        assert!(content.get(0).is_visible()); // 1. 0   -> 200, 200 < 500
        assert!(content.get(1).is_visible()); // 2. 200 -> 400, 200 < 500
        assert!(content.get(2).is_visible()); // 3. 400 -> 600, 400 < 500
        assert!(!content.get(3).is_visible()); // 4. 600 -> 800, 600 is NOT < 500, which means it is not visible.

        // Simulate the user dragging the scrollbar
        utils.push_event(TestEvent::Mouse {
            name: MouseEventName::MouseMove,
            cursor: (495., 20.).into(),
            button: Some(MouseButton::Left),
        });
        utils.wait_for_update().await;
        utils.push_event(TestEvent::Mouse {
            name: MouseEventName::MouseDown,
            cursor: (495., 20.).into(),
            button: Some(MouseButton::Left),
        });
        utils.wait_for_update().await;
        utils.push_event(TestEvent::Mouse {
            name: MouseEventName::MouseMove,
            cursor: (495., 320.).into(),
            button: Some(MouseButton::Left),
        });
        utils.wait_for_update().await;
        utils.push_event(TestEvent::Mouse {
            name: MouseEventName::MouseUp,
            cursor: (495., 320.).into(),
            button: Some(MouseButton::Left),
        });
        utils.wait_for_update().await;

        // Only the last three items are visible
        // Scrollview height is 500 but the user has dragged the scrollbar 300 pixels
        assert!(!content.get(0).is_visible()); // 1. 0   -> 200, 200 is NOT > 300, which means it is not visible.
        assert!(content.get(1).is_visible()); // 2. 200 -> 400, 400 > 300
        assert!(content.get(2).is_visible()); // 3. 400 -> 600, 600 > 300
        assert!(content.get(3).is_visible()); // 4. 600 -> 800, 800 > 300

        // Scroll up with arrows
        for _ in 0..5 {
            utils.push_event(TestEvent::Keyboard {
                name: KeyboardEventName::KeyDown,
                key: Key::ArrowUp,
                code: Code::ArrowUp,
                modifiers: Modifiers::default(),
            });
            utils.wait_for_update().await;
        }

        assert!(content.get(0).is_visible());
        assert!(content.get(1).is_visible());
        assert!(content.get(2).is_visible());
        assert!(!content.get(3).is_visible());

        // Scroll to the bottom with arrows
        utils.push_event(TestEvent::Keyboard {
            name: KeyboardEventName::KeyDown,
            key: Key::End,
            code: Code::End,
            modifiers: Modifiers::default(),
        });
        utils.wait_for_update().await;

        assert!(!content.get(0).is_visible());
        assert!(content.get(1).is_visible());
        assert!(content.get(2).is_visible());
        assert!(content.get(3).is_visible());
    }
}
