#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use freya::prelude::*;

fn main() {
    launch(app);
}

fn app() -> Element {
    rsx!(
        rect {
            content: "fit",
            wrap_content: "wrap",
            height: "100%",
            width: "100%",
            rect {
                width: "fill-min",
                height: "125",
                background: "rgb(71, 147, 175)",
            }
            rect {
                width: "25%",
                height: "125",
                background: "rgb(255, 196, 112)",
            }
            rect {
                width: "fill-min",
                height: "125",
                background: "rgb(221, 87, 70)",
            }
            rect {
                width: "300",
                height: "125",
                background: "rgb(139, 50, 44)",
            }
        }
    )
}
