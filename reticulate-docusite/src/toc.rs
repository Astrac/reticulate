use dioxus::{fullstack::http::Uri, prelude::*};

use crate::{
    app::{README, REPOSITORY},
    markdown::TocLink,
};

#[derive(Default, Clone, Debug, PartialEq, Props)]
pub struct TocProps {
    pub content: Vec<TocLink>,
}

impl TocProps {
    pub fn global() -> TocProps {
        let mut content = TocLink::from_document_heading(Uri::from_static("/readme"), README);

        content.append(&mut vec![
            TocLink {
                level: 1,
                text: "License".to_owned(),
                href: "/license".to_owned(),
            },
            TocLink {
                level: 1,
                text: "Repository".to_owned(),
                href: REPOSITORY.to_owned(),
            },
        ]);

        TocProps { content }
    }
}

#[component]
pub fn Toc(props: TocProps) -> Element {
    rsx! {
        aside {
            class: "toc",
            Nav { ..props }
        }
    }
}

#[component]
fn Nav(props: TocProps) -> Element {
    rsx! {
        h1 {
            class: "logo",
            "Reticulate"
        }
        nav {
            ul {
                for (index, link) in props.content.iter().enumerate() {
                    li {
                        class: "toc-link-{link.level}",
                        key: "toc-link-{index}",
                        a {
                            href: &link.href,
                            {link.text.as_str()}
                        }
                    }
                }
            }
        }
    }
}
