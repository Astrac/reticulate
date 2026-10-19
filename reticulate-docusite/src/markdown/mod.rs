mod toc_link;

pub use toc_link::TocLink;

use comrak::nodes::NodeValue;
use comrak::{Arena, Options, format_html, parse_document};

const DEMO_URL: &str = "https://reticulate.astrac.me/#demo";
const DEMO_HTML: &str = include_str!("../../assets/demo-frame.html");

pub fn page_options<'a>() -> Options<'a> {
    let mut options = Options::default();
    options.extension.table = true;
    options.extension.header_id_prefix = Some("".to_string());
    options.render.r#unsafe = true;
    options
}

pub fn to_html(markdown: &str) -> String {
    let arena = Arena::new();
    let options = page_options();

    let mut demo_targets = Vec::new();

    let root = parse_document(&arena, markdown, &options);

    for node in root.descendants() {
        let data = &node.data.borrow_mut().value;

        if let NodeValue::Link(link) = data
            && link.url == DEMO_URL
        {
            demo_targets.push(node);
        }
    }

    for target in demo_targets {
        let node_value = NodeValue::HtmlInline(DEMO_HTML.to_owned());
        let demo_html = arena.alloc(node_value.into());
        target.parent().unwrap().append(demo_html);
        target.detach();
    }

    let mut html = String::new();
    format_html(root, &options, &mut html).unwrap();
    html
}
