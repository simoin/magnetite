use actix_web::{get, http, web, HttpResponse};
use libxml::xpath::Context;
use rss::{Channel, Item};

use magnetite_cache::Storage;

use crate::util::ajson_get;
use crate::{
    error::{CustomError, Result},
    sites::{channel, item},
    util::{document, new_img_node, remove_node},
    CLIENT,
};

const BASE_URL: &str = "https://www.gcores.com";

// TODO remove all Option::unwrap, replace Error::unwrap with `?`

async fn get_item(url: String, title: String) -> Result<Item> {
    let item_url = format!("{}{}", BASE_URL, url);
    println!("{}", item_url);

    let api_url = format!("https://www.gcores.com/gapi/v1{}?include=media", url);
    let gapi_resp = CLIENT.get(&api_url).send().await?.text().await?;
    let article_resp = CLIENT.get(&item_url).send().await?.bytes().await?;

    let doc = document(article_resp)?;
    let context = Context::new(&doc).custom_err("create context failed")?;
    let images = {
        let attr_content = ajson::get(&gapi_resp, "data.attributes.content").unwrap();
        let attr_content = attr_content.as_str();

        let entity_map = ajson::get(attr_content, "entityMap").unwrap();
        let entity_map = entity_map.as_str();

        let entity_map_value = ajson::parse(entity_map).unwrap();
        entity_map_value
            .to_object()
            .values()
            .into_iter()
            .filter(|v| v.get("type").unwrap().as_str() == "IMAGE")
            .map(|img| {
                let caption = if let Some(cap) = img.get("data.caption") {
                    cap.as_str().to_owned()
                } else {
                    "".to_owned()
                };
                let src = if let Some(path) = img.get("data.path") {
                    path.as_str().to_owned()
                } else {
                    let src = img.get("data.src").unwrap();
                    src.as_str().to_owned()
                };
                let src = format!("https://image.gcores.com/{}", src);
                (caption, src)
            })
            .collect::<Vec<_>>()
    };

    let figures = context.evaluate("//figure").unwrap().get_nodes_as_vec();
    for (index, node) in figures.into_iter().enumerate() {
        if let Some(image) = images.get(index) {
            let mut parent = node.get_parent().unwrap();
            let img = new_img_node(&doc, &image.1)?;
            parent
                .replace_child_node(img, node)
                .custom_err("replace node failed")?;
        }
    }

    // remove md-editor-toolbar
    remove_node(&context, "//div[@class='md-editor-toolbar']")?;
    remove_node(&context, "//*[@class='story_hidden']")?;
    remove_node(&context, "//svg")?;

    let content = context
        .evaluate("//div[@class='story story-show']")
        .unwrap()
        .get_nodes_as_vec();

    let _ = ajson_get(&gapi_resp, "data.attributes.cover").map(|cover_url| {
        let _ = new_img_node(
            &doc,
            format!("https://image.gcores.com/{}", cover_url).as_str(),
        )
        .map(|mut cover_node| {
            content.get(0).map(|node| {
                node.get_first_child()
                    .map(|mut child| child.add_prev_sibling(&mut cover_node))
            });
        });
    });

    Ok(item(title, item_url, doc.node_to_string(&content[0])))
}

async fn get_channel(url: &str) -> Result<Channel> {
    println!("{}", url);
    let resp = CLIENT.get(url).send().await?.bytes().await?;
    let doc = document(resp)?;
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
    for node in item_node.iter().take(1) {
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

    Ok(channel(title, url.to_owned(), items))
}

#[get("/gcores/{category}")]
pub async fn gcores_handle(
    category: web::Path<(String,)>,
    storage: Storage,
) -> Result<HttpResponse> {
    println!("category: {:?}", category);
    let category = category.into_inner().0;
    let url = format!("{}/{}", BASE_URL, &category);
    let key = format!("/gcores/{}", &category);

    let channel = if let Some(channel) = storage.get::<_, Channel>(&key).await.unwrap() {
        channel
    } else {
        let channel = get_channel(&url).await?;
        storage.set(&key, &channel).await.unwrap();
        channel
    };

    Ok(HttpResponse::Ok()
        .append_header((http::header::CONTENT_TYPE, "application/xml"))
        .body(channel.to_string()))
    // channel.to_string().with_status(StatusCode::OK).with_header((http::header::CONTENT_TYPE, "application/xml"))
}
