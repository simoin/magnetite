pub mod http;
pub mod sites;

pub type Other = serde_json::Map<String, serde_json::Value>;
pub mod gcores {
    use crate::Other;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Root {
        pub data: Data,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Data {
        pub attributes: Attributes,
        #[serde(flatten)]
        other: Other,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Attributes {
        pub title: String,
        pub content: String,
        pub cover: Option<String>,
        pub thumb: String,
        #[serde(flatten)]
        other: Other,
    }
}
