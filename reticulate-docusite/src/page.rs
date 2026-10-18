use dioxus::prelude::*;

use crate::toc::{Toc, TocProps};

#[derive(Clone, PartialEq, Props)]
pub struct PageProps {
    children: Element,
    inner_html: String,
}

#[component]
pub fn Page(props: PageProps) -> Element {
    let toc_props = TocProps::global();
    let html = props.inner_html;

    rsx! {
        main {
            Toc { ..toc_props },
            section {
                class: "content",
                dangerous_inner_html: html
            }
        }
    }
}
