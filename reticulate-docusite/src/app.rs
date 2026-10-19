use dioxus::prelude::*;

use crate::{markdown, page::Page};

pub const LICENSE: &str = include_str!("../../LICENSE.md");
pub const README: &str = include_str!("../../README.md");
pub const REPOSITORY: &str = "https://github.com/Astrac/reticulate";

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");
const THEME_CSS: Asset = asset!("/assets/theme.css");

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[route("/")]
    Readme {},
    #[route("/license")]
    License {},
}

#[component]
pub fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        document::Link { rel: "stylesheet", href: THEME_CSS }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: "https://fontlibrary.org/face/fantasque-sans-mono", type: "text/css" }

        Router::<Route> {}
    }
}

#[component]
fn License() -> Element {
    let html = markdown::to_html(LICENSE);

    rsx! {
        Page {
            inner_html: html
        }
    }
}

#[component]
fn Readme() -> Element {
    let html = markdown::to_html(README);

    rsx! {
        Page {
            inner_html: html
        }
    }
}
