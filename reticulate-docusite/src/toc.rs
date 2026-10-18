use dioxus::prelude::*;

use crate::{
    app::{README, REPOSITORY, Route},
    markdown::TocLink,
};

#[derive(Debug, Default, Clone, PartialEq, Props)]
pub struct TocProps {
    pub content: Vec<TocLink>,
}

impl TocProps {
    pub fn global() -> TocProps {
        let mut content = TocLink::from_document_heading(Route::Readme {}.into(), README);

        content.append(&mut vec![
            TocLink {
                level: 1,
                text: "License".to_owned(),
                target: Route::License {}.to_string(),
            },
            TocLink {
                level: 1,
                text: "Repository".to_owned(),
                target: REPOSITORY.into(),
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
                for (index, TocLink { level, text, target }) in props.content.into_iter().enumerate() {
                    li {
                        class: "toc-link-{level}",
                        key: "toc-link-{index}",
                        a { href: target, {text} },
                    }
                }
            }
        }
    }
}
