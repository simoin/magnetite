use libxml::parser::Parser;
use libxml::tree::Node;
use libxml::xpath::Context;

use crate::{gcores, http};
use actix_web::{HttpResponse, Responder};
use chrono::Utc;
use rss::{Channel, ChannelBuilder, Item, ItemBuilder};

pub async fn gcores() -> impl Responder {
    let base_url = "https://www.gcores.com";

    let parser = Parser::default_html();
    let resp = http::get("https://www.gcores.com/articles").send().unwrap();
    let doc = parser.parse_string(resp.as_str().unwrap()).unwrap();
    let context = Context::new(&doc).unwrap();
    let articles_node = context
        .evaluate("//div[@class='original am_card original-normal original-article']")
        .unwrap()
        .get_nodes_as_vec();

    let items = articles_node
        .iter()
        .map(|node| {
            let url = node
                .findnodes(".//a[@class='original_imgArea_cover']/@href")
                .unwrap()
                .first()
                .unwrap()
                .get_content();
            let title = node
                .findnodes(".//a[@class='am_card_content original_content']/h3/text()")
                .unwrap()
                .first()
                .unwrap()
                .get_content();
            (url, title)
        })
        .map(|(url, title)| {
            let article_url = format!("{}{}", base_url, url);
            println!("{}", article_url);
            let api_url = format!("https://www.gcores.com/gapi/v1{}?include=media", url);
            let resp = http::get(api_url.as_str()).send().unwrap();

            let json = resp.json::<gcores::Root>().unwrap();

            let parser = Parser::default_html();
            let article_resp = http::get(&article_url).send().unwrap();
            let article_resp = article_resp.as_str().unwrap();
            let doc = parser.parse_string(article_resp).unwrap();
            let context = Context::new(&doc).unwrap();

            let json: serde_json::Value =
                serde_json::from_str(&json.data.attributes.content).unwrap();
            let entity_map = if let serde_json::Value::Object(map) = json {
                map.get("entityMap").unwrap().clone()
            } else {
                panic!()
            };
            let images: Vec<(String, String)> =
                entity_map
                    .as_object()
                    .unwrap()
                    .values()
                    .fold(Vec::new(), |mut v, value| {
                        if value
                            .pointer("/type")
                            .unwrap()
                            .as_str()
                            .unwrap()
                            .eq("IMAGE")
                        {
                            let cap = if let Some(c) = value.pointer("/data/caption") {
                                c.as_str().unwrap()
                            } else {
                                ""
                            };
                            let src = if let Some(path) = value.pointer("/data/path") {
                                format!("https://image.gcores.com/{}", path.as_str().unwrap())
                            } else {
                                value
                                    .pointer("/data/src")
                                    .unwrap()
                                    .as_str()
                                    .unwrap()
                                    .to_owned()
                            };
                            v.push((cap.to_owned(), src));
                        }
                        v
                    });

            context
                .evaluate("//figure")
                .unwrap()
                .get_nodes_as_vec()
                .into_iter()
                .enumerate()
                .for_each(|(index, node)| {
                    if let Some(image) = images.get(index) {
                        let mut parent = node.get_parent().unwrap();
                        let mut new_node = Node::new("img", None, &doc).unwrap();
                        new_node.set_attribute("src", &image.1).unwrap();
                        parent.replace_child_node(new_node, node).unwrap();
                    }
                });

            // remove md-editor-toolbar
            context
                .evaluate("//div[@class='md-editor-toolbar']")
                .unwrap()
                .get_nodes_as_vec()
                .into_iter()
                .for_each(|mut node| node.unlink());
            context
                .evaluate("//*[@class='story_hidden']")
                .unwrap()
                .get_nodes_as_vec()
                .into_iter()
                .for_each(|mut node| node.unlink());

            let content = context
                .evaluate("//div[@class='story story-show']")
                .unwrap()
                .get_nodes_as_vec();

            let item = ItemBuilder::default()
                .title(title)
                .link(article_url)
                .description(format!("{}", doc.node_to_string(&content[0])))
                .build()
                .unwrap();
            item
        })
        .collect::<Vec<Item>>();

    let channel: Channel = ChannelBuilder::default()
        .title("Gcores")
        .link(base_url)
        .description("Rsshub-RS")
        .language("zh-cn".to_string())
        .generator("Rsshub-RS".to_string())
        .ttl("5".to_string())
        .last_build_date(Utc::now().to_rfc2822())
        .items(items)
        .build()
        .unwrap();

    HttpResponse::Ok()
        .header("content-type", "application/xml")
        .body(channel.to_string())
}
