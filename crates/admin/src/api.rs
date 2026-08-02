pub mod admin;

use gloo_net::http::Request;
use serde::de::DeserializeOwned;

pub async fn api_get<T: DeserializeOwned>(path: &str, token: Option<&str>) -> Option<T> {
    let url = format!("http://localhost:8000{}", path);
    let mut builder = Request::get(&url);
    if let Some(t) = token {
        builder = builder.header("Authorization", &format!("Bearer {}", t));
    }
    let resp = builder.send().await.ok()?;
    resp.json::<T>().await.ok()
}

pub async fn api_post<T: DeserializeOwned>(path: &str, body: &serde_json::Value, token: Option<&str>) -> Option<T> {
    let url = format!("http://localhost:8000{}", path);
    let mut builder = Request::post(&url).header("Content-Type", "application/json");
    if let Some(t) = token {
        builder = builder.header("Authorization", &format!("Bearer {}", t));
    }
    let builder = builder.json(body).ok()?;
    let resp = builder.send().await.ok()?;
    resp.json::<T>().await.ok()
}

pub async fn api_post_status(path: &str, body: &serde_json::Value, token: Option<&str>) -> bool {
    let url = format!("http://localhost:8000{}", path);
    let mut builder = Request::post(&url).header("Content-Type", "application/json");
    if let Some(t) = token {
        builder = builder.header("Authorization", &format!("Bearer {}", t));
    }
    let builder = match builder.json(body) {
        Ok(b) => b,
        Err(_) => return false,
    };
    match builder.send().await {
        Ok(resp) => resp.ok(),
        Err(_) => false,
    }
}
