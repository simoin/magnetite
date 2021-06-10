use actix_web::{get, http, web, HttpResponse};
use log::{debug, error, info};
use rss::{Channel, Item};

use magnetite_cache::Storage;

use crate::{
    error::{Error, Result},
    sites::{channel, item},
    util::ajson_get,
    xpath::Document,
    CLIENT,
};

const BASE_URL: &str = "https://www.gcores.com";

fn get_images_info(json: &str) -> Option<Vec<(String, String)>> {
    let entity_map = ajson::get(&json, "data.attributes.content")
        .map(|val| ajson::get(val.as_str(), "entityMap").map(|val| val.to_string()))
        .flatten();

    entity_map
        .map(|entity_map| {
            ajson::parse(&entity_map).map(|value| {
                value
                    .to_object()
                    .values()
                    .into_iter()
                    .filter(|v| v.get("type").map_or(false, |val| val.as_str() == "IMAGE"))
                    .map(|img| {
                        let caption = img
                            .get("data.caption")
                            .map_or("".to_string(), |cap| cap.to_string());
                        let src = img.get("data.path").map_or_else(
                            || {
                                img.get("data.src")
                                    .map_or("".to_string(), |src| src.to_string())
                            },
                            |path| path.to_string(),
                        );
                        (caption, format!("https://image.gcores.com/{}", src))
                    })
                    .collect::<Vec<_>>()
            })
        })
        .flatten()
}

async fn get_item(url: String, title: String) -> Result<Item> {
    let item_url = format!("{}{}", BASE_URL, url);
    debug!(target: "get_item", "item_url: {}", item_url);

    let api_url = format!("https://www.gcores.com/gapi/v1{}?include=media", url);
    let gapi_resp = CLIENT.get(&api_url).send().await?.text().await?;
    let article_resp = CLIENT.get(&item_url).send().await?.bytes().await?;

    let doc = Document::from_bytes(article_resp)?;

    let images = get_images_info(&gapi_resp);
    if let Some(images) = images {
        let figures = doc.evaluate("//figure")?;
        for (index, mut node) in figures.into_iter().enumerate() {
            if let Some(((cap, url), mut parent)) = images.get(index).zip(node.get_parent()) {
                let mut text = doc.create_node("p")?;
                text.set_content(cap)?;
                node.add_next_sibling(&mut text)?;

                let mut img = doc.create_node("img")?;
                img.set_attribute("src", url)?;
                parent.replace_child_node(img, node)?;
            }
        }
    }

    // remove md-editor-toolbar
    doc.remove_node("//div[@class='md-editor-toolbar']")?;
    doc.remove_node("//*[@class='story_hidden']")?;
    doc.remove_node("//svg")?;

    let content = doc.evaluate("//div[@class='story story-show']")?;

    ajson_get(&gapi_resp, "data.attributes.cover")
        .map(|cover_url| {
            doc.create_node("img").map(|mut img| {
                img.set_attribute(
                    "src",
                    format!("https://image.gcores.com/{}", cover_url).as_str(),
                )
                .and_then(|_| {
                    content.get(0).map(|node| {
                        node.get_first_child()
                            .map(|mut child| child.add_prev_sibling(&mut img))
                    });
                    Ok(())
                })
            })
        })
        .transpose()?;

    Ok(item(title, item_url, doc.node_to_string(&content[0])))
}

async fn get_channel(url: &str) -> Result<Channel> {
    debug!(target: "get_channel", "url: {}", url);
    let resp = CLIENT.get(url).send().await?.bytes().await?;

    let doc = Document::from_bytes(resp)?;
    // let doc = document(resp)?;
    // let context = Context::new(&doc).custom_err("create context failed")?;

    let title = doc
        .evaluate("//title")?
        .first()
        .map(|node| node.content())
        .ok_or(Error::LibXMLError("get channel title failed".to_string()))?;

    let item_node =
        doc.evaluate("//div[contains(@class,'original-normal') and contains(@class,'am_card')]")?;

    let mut items = Vec::new();
    for node in item_node.iter() {
        let url = node
            .find_nodes(".//a[@class='original_imgArea_cover']/@href")?
            .first()
            .map(|node| node.content())
            .ok_or(Error::LibXMLError("get item url failed".to_string()))?;
        let title = node
            .find_nodes(".//a[@class='am_card_content original_content']/h3/text()")?
            .first()
            .map(|node| node.content())
            .ok_or(Error::LibXMLError("get item title failed".to_string()))?;

        let item = get_item(url, title).await?;
        items.push(item);
    }

    Ok(channel(title, url.to_owned(), items))
}

#[get("/gcores/{category}")]
pub async fn gcores_handle(
    category: web::Path<(String,)>,
    storage: Storage,
) -> Result<HttpResponse> {
    debug!(target: "gcores_handle", "category: {:?}", category);
    let category = category.into_inner().0;
    let url = format!("{}/{}", BASE_URL, &category);
    let key = format!("/gcores/{}", &category);

    let channel = if let Some(channel) = storage.get::<_, Channel>(&key).await? {
        channel
    } else {
        let channel = get_channel(&url).await?;
        storage.set(&key, &channel).await?;
        channel
    };

    Ok(HttpResponse::Ok()
        .append_header((http::header::CONTENT_TYPE, "application/xml"))
        .body(channel.to_string()))
    // channel.to_string().with_status(StatusCode::OK).with_header((http::header::CONTENT_TYPE, "application/xml"))
}
