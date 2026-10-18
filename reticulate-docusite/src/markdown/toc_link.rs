use comrak::nodes::{NodeHeading, NodeValue};
use comrak::{Anchorizer, Arena, parse_document};
use dioxus::router::NavigationTarget;

use crate::app::Route;
use crate::markdown::page_options;

#[derive(Clone, Debug, PartialEq)]
pub struct TocLink {
    pub level: u8,
    pub text: String,
    pub target: String,
}

impl TocLink {
    pub fn from_document_heading(
        document_nav: NavigationTarget<Route>,
        markdown: &str,
    ) -> Vec<TocLink> {
        let arena = Arena::new();
        let mut anchorizer = Anchorizer::new();
        let options = page_options();

        let root = parse_document(&arena, markdown, &options);

        let mut headings = Vec::new();
        for node in root.descendants() {
            if let NodeValue::Heading(NodeHeading { level, .. }) = node.data.borrow().value {
                let mut text = String::new();
                for child in node.descendants() {
                    if let NodeValue::Text(ref t) = child.data.borrow().value {
                        text.push_str(t);
                    }
                }

                let route = match &document_nav {
                    NavigationTarget::Internal(s) => s.to_string(),
                    NavigationTarget::External(r) => r.clone(),
                };

                let target = format!("{}#{}", route, anchorizer.anchorize(&text));

                headings.push(TocLink {
                    level,
                    text,
                    target,
                });
            }
        }

        headings
    }
}
