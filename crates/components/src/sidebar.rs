use dioxus::prelude::*;
use freya_core::platform::CursorIcon;
use freya_elements as dioxus_elements;
use freya_hooks::{
    use_activable_route,
    use_applied_theme,
    use_platform,
    SidebarItemTheme,
    SidebarItemThemeWith,
    SidebarTheme,
    SidebarThemeWith,
};

use crate::{
    ButtonStatus,
    ScrollView,
};

#[allow(non_snake_case)]
#[component]
pub fn Sidebar(
    /// Theme override.
    theme: Option<SidebarThemeWith>,
    /// This is what is rendered next to the sidebar.
    children: Element,
    /// This is what is rendered in the sidebar.
    sidebar: Element,
    /// Width of the sidebar.
    #[props(default = "180".to_string())]
    width: String,
) -> Element {
    let SidebarTheme {
        spacing,
        font_theme,
        background,
    } = use_applied_theme!(&theme, sidebar);

    rsx!(
        rect {
            width: "100%",
            height: "100%",
            direction: "horizontal",
            rect {
                overflow: "clip",
                width,
                height: "100%",
                background: "{background}",
                color: "{font_theme.color}",
                shadow: "2 0 5 0 rgb(0, 0, 0, 30)",
                ScrollView {
                    padding: "8",
                    spacing,
                    {sidebar}
                }
            }
            rect {
                overflow: "clip",
                width: "fill",
                height: "100%",
                color: "{font_theme.color}",
                {children}
            }
        }
    )
}

#[allow(non_snake_case)]
#[component]
pub fn SidebarItem(
    /// Theme override.
    theme: Option<SidebarItemThemeWith>,
    /// Inner content for the SidebarItem.
    children: Element,
    /// Optionally handle the `onpress` event in the SidebarItem.
    onpress: Option<EventHandler<()>>,
    /// Optionally specify a custom `overflow` attribute for this component. Defaults to `clip`.
    #[props(default = "clip".to_string())]
    overflow: String,
) -> Element {
    let SidebarItemTheme {
        margin,
        hover_background,
        background,
        corner_radius,
        padding,
        font_theme,
    } = use_applied_theme!(&theme, sidebar_item);
    let mut status = use_signal(ButtonStatus::default);
    let platform = use_platform();
    let is_active = use_activable_route();

    use_drop(move || {
        if *status.read() == ButtonStatus::Hovering {
            platform.set_cursor(CursorIcon::default());
        }
    });

    let onclick = move |_| {
        if let Some(onpress) = &onpress {
            onpress.call(());
        }
    };

    let onmouseenter = move |_| {
        platform.set_cursor(CursorIcon::Pointer);
        status.set(ButtonStatus::Hovering);
    };

    let onmouseleave = move |_| {
        platform.set_cursor(CursorIcon::default());
        status.set(ButtonStatus::default());
    };

    let background = match *status.read() {
        _ if is_active => hover_background,
        ButtonStatus::Hovering => hover_background,
        ButtonStatus::Idle => background,
    };

    rsx!(
        rect {
            overflow,
            margin: "{margin}",
            onclick,
            onmouseenter,
            onmouseleave,
            width: "100%",
            height: "auto",
            color: "{font_theme.color}",
            corner_radius: "{corner_radius}",
            padding: "{padding}",
            background: "{background}",
            {children}
        }
    )
}
