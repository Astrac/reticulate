use comrak::nodes::{NodeHeading, NodeValue};
use comrak::{Anchorizer, Arena, parse_document};
use dioxus::fullstack::http::Uri;

use crate::markdown::page_options;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TocLink {
    pub level: u8,
    pub text: String,
    pub href: String,
}

impl TocLink {
    pub fn from_document_heading(uri: Uri, markdown: &str) -> Vec<TocLink> {
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

                let href = format!("{uri}#{}", anchorizer.anchorize(&text));

                headings.push(TocLink { level, href, text });
            }
        }

        headings
    }
}
