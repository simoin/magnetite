use libxml::{
    parser::Parser,
    tree::{Document, Node},
    xpath::Context,
};

use crate::error::{Error, Result};

pub fn ajson_get(json: &str, xpath: &str) -> Option<String> {
    ajson::get(json, xpath).map(|val| val.to_string())
}

pub fn remove_node(ctx: &Context, xpath: &str) -> Result<()> {
    ctx.evaluate(xpath)
        .map(|obj| {
            obj.get_nodes_as_vec()
                .into_iter()
                .for_each(|mut node| node.unlink())
        })
        .map_err(|_| Error::LibXMLError(format!("remove node error, path: {}", xpath)))
}

pub fn document<Bytes: AsRef<[u8]>>(input: Bytes) -> Result<Document> {
    let parser = Parser::default_html();
    Ok(parser.parse_string(input)?)
}

pub fn img_node(doc: &Document, src: &str) -> Result<Node> {
    let mut img = Node::new("img", None, &doc)
        .map_err(|_| Error::LibXMLError(format!("create image node error, src: {}", src)))?;
    img.set_attribute("src", src)
        .map_err(|_| Error::LibXMLError("image node set attr failed".to_string()))?;
    Ok(img)
}

pub fn text_node(doc: &Document, content: &str) -> Result<Node> {
    let mut text = Node::new("p", None, &doc)
        .map_err(|_| Error::LibXMLError(format!("create text node error, src: {}", content)))?;
    text.set_content(content)
        .map_err(|_| Error::LibXMLError("text node set attr failed".to_string()))?;
    Ok(text)
}
