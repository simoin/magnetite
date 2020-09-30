use actix_web::{get, http, web, HttpResponse};
use anyhow::Result;
use libxml::{parser::Parser, tree::Node, xpath::Context};
use rss::{Guid, Item, ItemBuilder};
use serde::{Deserialize, Serialize};

use crate::cache::CACHE;
use crate::error::Error;
use crate::sites::channel;
use crate::{sites::Other, util::remove_node, CLIENT};

const BASE_URL: &str = "https://www.gcores.com";

#[derive(Debug, Serialize, Deserialize)]
struct GApiResp {
    pub data: Data,
}

#[derive(Debug, Serialize, Deserialize)]
struct Data {
    pub attributes: Attributes,
    #[serde(flatten)]
    other: Other,
}

#[derive(Debug, Serialize, Deserialize)]
struct Attributes {
    pub title: String,
    pub content: String,
    pub cover: Option<String>,
    pub thumb: String,
    #[serde(flatten)]
    other: Other,
}

pub async fn get_item(
    url: String,
    title: String,
) -> std::result::Result<Item, Box<dyn std::error::Error>> {
    let item_url = format!("{}{}", BASE_URL, url);
    println!("{}", item_url);
    let api_url = format!("https://www.gcores.com/gapi/v1{}?include=media", url);
    let gapi_resp = CLIENT
        .get(&api_url)
        .send()
        .await?
        .json::<GApiResp>()
        .await?;

    let parser = Parser::default_html();
    let article_resp = CLIENT
        .get(&item_url)
        .send()
        .await
        .unwrap()
        .bytes()
        .await
        .unwrap();
    let doc = parser.parse_string(article_resp).unwrap();
    let context = Context::new(&doc).unwrap();

    let attr_content: serde_json::Value =
        serde_json::from_str(&gapi_resp.data.attributes.content).unwrap();
    let entity_map = if let serde_json::Value::Object(map) = attr_content {
        map.get("entityMap").unwrap().to_owned()
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

    let figures = context.evaluate("//figure").unwrap().get_nodes_as_vec();
    for (index, node) in figures.into_iter().enumerate() {
        if let Some(image) = images.get(index) {
            let mut parent = node.get_parent().unwrap();
            let mut new_node = Node::new("img", None, &doc).unwrap();
            new_node.set_attribute("src", &image.1)?;
            parent.replace_child_node(new_node, node)?;
        }
    }

    // remove md-editor-toolbar
    remove_node(&context, "//div[@class='md-editor-toolbar']");
    remove_node(&context, "//*[@class='story_hidden']");
    remove_node(&context, "//svg");

    let content = context
        .evaluate("//div[@class='story story-show']")
        .unwrap()
        .get_nodes_as_vec();

    let mut guid = Guid::default();
    guid.set_permalink(false);
    guid.set_value(&item_url);

    Ok(ItemBuilder::default()
        .title(title)
        .link(item_url)
        .description(format!("{}", doc.node_to_string(&content[0])))
        .guid(guid)
        .build()
        .unwrap())
}

#[get("/gcores/{category}")]
pub async fn gcores(category: web::Path<(String,)>) -> Result<HttpResponse, Error> {
    println!("{:?}", category);
    let url = format!("{}/{}", BASE_URL, category.into_inner().0);
    if let Some(channel) = CACHE.try_get(&url) {
        eprintln!("got cache");
        return Ok(HttpResponse::Ok()
            .header(http::header::CONTENT_TYPE, "application/xml")
            .body(channel.to_string()));
    }
    let resp = CLIENT.get(&url).send().await?.bytes().await?;
    let parser = Parser::default_html();
    let doc = parser.parse_string(resp)?;
    let context = Context::new(&doc).unwrap();
    let item_node = context
        .evaluate("//div[contains(@class,'original-normal') and contains(@class,'am_card')]")
        .unwrap()
        .get_nodes_as_vec();

    let title = context
        .evaluate("//title")
        .unwrap()
        .get_nodes_as_vec()
        .first()
        .unwrap()
        .get_content();

    let mut items = Vec::new();
    for node in item_node {
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

        let item = get_item(url, title).await.unwrap();
        items.push(item);
    }

    let channel = channel(title, url.clone(), items);

    CACHE.set(&url, &channel);

    Ok(HttpResponse::Ok()
        .header(http::header::CONTENT_TYPE, "application/xml")
        .body(channel.to_string()))
}
