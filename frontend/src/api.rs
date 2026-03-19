use gloo_net::http::Request;
use serde::{de::DeserializeOwned, Serialize};
use crate::error::AppError;
use crate::types::ErrorResponse;

const API_BASE: &str = "http://localhost:8080";

async fn get_token() -> Option<String> {
    use gloo_storage::{LocalStorage, Storage};
    LocalStorage::get::<String>(crate::store::TOKEN_KEY).ok()
}

pub async fn get<T: DeserializeOwned>(path: &str) -> Result<T, AppError> {
    let url = format!("{}{}", API_BASE, path);
    let mut req = Request::get(&url);
    if let Some(token) = get_token().await {
        req = req.header("Authorization", &format!("Bearer {}", token));
    }
    let resp = req.send().await.map_err(|e| AppError::Network(e.to_string()))?;
    let status = resp.status();
    if status == 401 {
        return Err(AppError::Auth("認証が必要です".to_string()));
    }
    if status == 403 {
        return Err(AppError::Auth("権限がありません".to_string()));
    }
    if !resp.ok() {
        let err: ErrorResponse = resp.json().await.unwrap_or(ErrorResponse { error: "サーバーエラー".to_string() });
        return Err(AppError::Server(err.error));
    }
    resp.json::<T>().await.map_err(|e| AppError::Network(e.to_string()))
}

pub async fn post<B: Serialize, T: DeserializeOwned>(path: &str, body: &B) -> Result<T, AppError> {
    let url = format!("{}{}", API_BASE, path);
    let mut req = Request::post(&url).header("Content-Type", "application/json");
    if let Some(token) = get_token().await {
        req = req.header("Authorization", &format!("Bearer {}", token));
    }
    let resp = req.json(body).map_err(|e| AppError::Network(e.to_string()))?
        .send().await.map_err(|e| AppError::Network(e.to_string()))?;
    let status = resp.status();
    if status == 401 {
        return Err(AppError::Auth("認証が必要です".to_string()));
    }
    if status == 403 {
        return Err(AppError::Auth("権限がありません".to_string()));
    }
    if !resp.ok() {
        let err: ErrorResponse = resp.json().await.unwrap_or(ErrorResponse { error: "サーバーエラー".to_string() });
        return Err(AppError::Server(err.error));
    }
    resp.json::<T>().await.map_err(|e| AppError::Network(e.to_string()))
}

pub async fn post_no_auth<B: Serialize, T: DeserializeOwned>(path: &str, body: &B) -> Result<T, AppError> {
    let url = format!("{}{}", API_BASE, path);
    let resp = Request::post(&url)
        .header("Content-Type", "application/json")
        .json(body).map_err(|e| AppError::Network(e.to_string()))?
        .send().await.map_err(|e| AppError::Network(e.to_string()))?;
    if !resp.ok() {
        let err: ErrorResponse = resp.json().await.unwrap_or(ErrorResponse { error: "サーバーエラー".to_string() });
        return Err(AppError::Server(err.error));
    }
    resp.json::<T>().await.map_err(|e| AppError::Network(e.to_string()))
}

pub async fn patch<B: Serialize, T: DeserializeOwned>(path: &str, body: &B) -> Result<T, AppError> {
    let url = format!("{}{}", API_BASE, path);
    let mut req = Request::patch(&url).header("Content-Type", "application/json");
    if let Some(token) = get_token().await {
        req = req.header("Authorization", &format!("Bearer {}", token));
    }
    let resp = req.json(body).map_err(|e| AppError::Network(e.to_string()))?
        .send().await.map_err(|e| AppError::Network(e.to_string()))?;
    let status = resp.status();
    if status == 401 {
        return Err(AppError::Auth("認証が必要です".to_string()));
    }
    if status == 403 {
        return Err(AppError::Auth("権限がありません".to_string()));
    }
    if !resp.ok() {
        let err: ErrorResponse = resp.json().await.unwrap_or(ErrorResponse { error: "サーバーエラー".to_string() });
        return Err(AppError::Server(err.error));
    }
    resp.json::<T>().await.map_err(|e| AppError::Network(e.to_string()))
}

pub async fn delete_req(path: &str) -> Result<(), AppError> {
    let url = format!("{}{}", API_BASE, path);
    let mut req = Request::delete(&url);
    if let Some(token) = get_token().await {
        req = req.header("Authorization", &format!("Bearer {}", token));
    }
    let resp = req.send().await.map_err(|e| AppError::Network(e.to_string()))?;
    let status = resp.status();
    if status == 401 {
        return Err(AppError::Auth("認証が必要です".to_string()));
    }
    if status == 403 {
        return Err(AppError::Auth("権限がありません".to_string()));
    }
    if !resp.ok() {
        return Err(AppError::Server("削除に失敗しました".to_string()));
    }
    Ok(())
}

pub async fn post_multipart<T: DeserializeOwned>(path: &str, form_data: web_sys::FormData) -> Result<T, AppError> {
    use wasm_bindgen::JsValue;
    let url = format!("{}{}", API_BASE, path);
    let token = get_token().await;

    let mut req = Request::post(&url);
    if let Some(t) = token {
        req = req.header("Authorization", &format!("Bearer {}", t));
    }

    let resp = req
        .body(JsValue::from(form_data))
        .map_err(|e| AppError::Network(format!("{:?}", e)))?
        .send()
        .await
        .map_err(|e| AppError::Network(e.to_string()))?;

    let status = resp.status();
    if status == 401 {
        return Err(AppError::Auth("認証が必要です".to_string()));
    }
    if !resp.ok() {
        let err: ErrorResponse = resp.json().await.unwrap_or(ErrorResponse { error: "サーバーエラー".to_string() });
        return Err(AppError::Server(err.error));
    }
    resp.json::<T>().await.map_err(|e| AppError::Network(e.to_string()))
}
