use dashmap::DashMap;

struct AppState {
    redis: Option<String>,
    ttl: usize,
    env: DashMap<String, String>
}