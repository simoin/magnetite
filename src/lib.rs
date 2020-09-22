mod services;
mod util;
mod sites;

use sites::gcores::gcores;

pub use services::rss_service;

use lazy_static::lazy_static;
use reqwest::Client;



const UA: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_13_4) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/65.0.3325.181 Safari/537.36";

lazy_static! {
    static ref CLIENT: Client = reqwest::Client::builder().user_agent(UA).build().unwrap();
}
