// use actix_web::{get, http, web, HttpResponse};
// use anyhow::Result;
// use reqwest::header::{HeaderMap, REFERER};
// use rss::{Channel, Item};
//
// use crate::cache::CACHE;
// use crate::error::Error;
// use crate::sites::{channel, item};
// use crate::util::ajson_get_string;
// use crate::CLIENT;
//
// async fn get_item(
//     url: String, title: String,
// ) -> std::result::Result<Item, Box<dyn std::error::Error>> {
//     unimplemented!()
// }
//
// async fn get_channel(
//     url: &str, headers: HeaderMap, username: String,
// ) -> std::result::Result<Channel, Error> {
//     let resp = CLIENT.get(url).headers(headers).send().await?.text().await?;
//     let items = get_item(url.to_owned(), "".to_owned()).await.unwrap();
//
//     Ok(channel(
//         format!("{}的微博", &username),
//         url.to_owned(),
//         vec![items],
//     ))
//
//     // unimplemented!()
// }
//
// #[get("/user/{uid}")]
// pub async fn weibo_user(uid: web::Path<(String,)>) -> Result<HttpResponse, Error> {
//     let base_url = "https://m.weibo.cn/api/container/getIndex?type=uid&value=";
//     println!("{:?}", uid);
//     let uid = uid.into_inner().0;
//     let container_api_url = format!("{}{}", base_url, &uid);
//     let container = CLIENT
//         .get(&container_api_url)
//         .header(REFERER, "https://m.weibo.cn/")
//         .send()
//         .await?
//         .text()
//         .await?;
//
//     let username = ajson_get_string(&container, "data.userInfo.screen_name");
//     let container_id = ajson_get_string(&container, "data.tabsInfo.tabs.1.containerid");
//
//     let cards_url = format!("{}&containerid={}", container_api_url, container_id);
//     let channel = if let Some(channel) = CACHE.get_channel(&cards_url) {
//         channel
//     } else {
//         let headers = {
//             let mut headers = HeaderMap::with_capacity(3);
//             headers.insert(
//                 REFERER,
//                 format!("https://m.weibo.cn/u/{}", &uid).parse().unwrap(),
//             );
//             headers.insert("MWeibo-Pwa", "1".parse().unwrap());
//             headers.insert("X-Requested-With", "XMLHttpRequest".parse().unwrap());
//             headers
//         };
//
//         let channel = get_channel(&cards_url, headers, username).await?;
//         CACHE.set_channel(&cards_url, &channel);
//         channel
//     };
//
//     Ok(HttpResponse::Ok()
//         .header(http::header::CONTENT_TYPE, "application/xml")
//         .body(channel.to_string()))
// }
