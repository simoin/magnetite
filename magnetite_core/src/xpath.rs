use libxml::parser::Parser;

use crate::error::{CustomError, Result};

pub(crate) struct Document {
    doc: libxml::tree::Document,
    ctx: libxml::xpath::Context,
}

impl Document {
    pub fn from_bytes<Bytes: AsRef<[u8]>>(input: Bytes) -> Result<Document> {
        let parser = Parser::default_html();
        let doc = parser.parse_string(input)?;
        let cxt = libxml::xpath::Context::new(&doc).custom_err("create context failed")?;

        Ok(Document { doc, ctx: cxt })
    }

    pub fn node_to_string(&self, node: &Node) -> String {
        self.doc.node_to_string(&node.0)
    }

    pub fn evaluate(&self, xpath: &str) -> Result<Vec<Node>> {
        let res = self
            .ctx
            .evaluate(xpath)
            .custom_err(format!("evaluate error, xpath: {}", xpath).as_str())?
            .get_nodes_as_vec()
            .into_iter()
            .map(|node| Node(node))
            .collect();
        Ok(res)
    }

    pub fn remove_node(&self, xpath: &str) -> Result<()> {
        self.ctx
            .evaluate(xpath)
            .map(|obj| {
                obj.get_nodes_as_vec()
                    .into_iter()
                    .for_each(|mut node| node.unlink())
            })
            .custom_err(&format!("remove node error, path: {}", xpath))
    }

    pub fn create_node(&self, tag: &str) -> Result<Node> {
        let node = libxml::tree::Node::new(tag, None, &self.doc).custom_err("create node error")?;
        Ok(Node(node))
    }
}

pub(crate) struct Node(libxml::tree::Node);

impl Node {
    pub fn set_attribute(&mut self, name: &str, value: &str) -> Result<()> {
        self.0
            .set_attribute(name, value)
            .custom_err(format!("set_attribute error: name: {}, value: {}", name, value).as_ref())
    }

    pub fn set_content(&mut self, content: &str) -> Result<()> {
        self.0
            .set_content(content)
            .custom_err(format!("set_content error: content: {}", content).as_ref())
    }

    pub fn find_nodes(&self, xpath: &str) -> Result<Vec<Node>> {
        self.0
            .findnodes(xpath)
            .custom_err(format!("find_nodes error: xpath: {}", xpath).as_ref())
            .and_then(|nodes| Ok(nodes.into_iter().map(|node| Node(node)).collect()))
    }

    pub fn content(&self) -> String {
        self.0.get_content()
    }

    pub fn add_next_sibling(&mut self, node: &mut Node) -> Result<()> {
        self.0
            .add_next_sibling(&mut node.0)
            .custom_err("add_next_sibling failed")
    }

    pub fn add_prev_sibling(&mut self, node: &mut Node) -> Result<()> {
        self.0
            .add_prev_sibling(&mut node.0)
            .custom_err("add_prev_sibling failed")
    }

    pub fn get_parent(&self) -> Option<Self> {
        self.0.get_parent().map(|node| Node(node))
    }

    pub fn get_first_child(&self) -> Option<Self> {
        self.0.get_first_child().map(|node| Node(node))
    }

    pub fn replace_child_node(&mut self, new: Node, old: Node) -> Result<()> {
        self.0
            .replace_child_node(new.0, old.0)
            .map(|_| ())
            .custom_err("replace_child_node failed")
    }
}
