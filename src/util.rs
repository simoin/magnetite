use libxml::parser::Parser;
use libxml::tree::{Document, Node};
use libxml::xpath::Context;

pub fn remove_node(ctx: &Context, xpath: &str) {
    ctx.evaluate(xpath)
        .unwrap()
        .get_nodes_as_vec()
        .into_iter()
        .for_each(|mut node| node.unlink());
}

pub fn doc<Bytes: AsRef<[u8]>>(input: Bytes) -> Document {
    let parser = Parser::default_html();
    parser.parse_string(input).unwrap()
}

pub fn new_img_node(
    doc: &Document,
    src: &str,
) -> std::result::Result<Node, Box<dyn std::error::Error>> {
    let mut img = Node::new("img", None, &doc).unwrap();
    img.set_attribute("src", src)?;
    Ok(img)
}
