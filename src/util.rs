use libxml::xpath::Context;

pub fn remove_node(ctx: &Context, xpath: &str) {
    ctx.evaluate(xpath)
        .unwrap()
        .get_nodes_as_vec()
        .into_iter()
        .for_each(|mut node| node.unlink());
}
