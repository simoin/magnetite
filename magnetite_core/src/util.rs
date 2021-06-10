pub fn ajson_get(json: &str, xpath: &str) -> Option<String> {
    ajson::get(json, xpath).map(|val| val.to_string())
}
