use crate::Result;
use crate::error::Error::{self, MissingHeader};
use crate::hash::gen_request_hash;
use crate::model::ChatRequest;
use crate::process::ChatProcess;
use crate::serve::AppState;
use axum::{
    Json,
    extract::State,
    response::{IntoResponse, Response},
};
use axum_extra::{
    TypedHeader,
    extract::WithRejection,
    headers::{Authorization, authorization::Bearer},
};
use reqwest::{Client, header};

const ORIGIN_API: &str = "https://duck.ai";

pub async fn models(
    State(state): State<AppState>,
    bearer: Option<TypedHeader<Authorization<Bearer>>>,
) -> crate::Result<Response> {
    state.valid_key(bearer)?;

    let model_data = vec![
        serde_json::json!({
            "id": "gpt-4o-mini",
            "object": "model",
            "created": 1686935002,
            "owned_by": "openai",
        }),
        serde_json::json!({
            "id": "gpt-5-mini",
            "object": "model",
            "created": 1686935002,
            "owned_by": "openai",
        }),
        serde_json::json!({
            "id": "gpt-oss-120b",
            "object": "model",
            "created": 1686935002,
            "owned_by": "openai",
        }),
        serde_json::json!({
            "id": "llama-4-scout",
            "object": "model",
            "created": 1686935002,
            "owned_by": "meta",
        }),
        serde_json::json!({
            "id": "claude-3.5-haiku",
            "object": "model",
            "created": 1686935002,
            "owned_by": "claude",
        }),
        serde_json::json!({
            "id": "mixtral-small-3",
            "object": "model",
            "created": 1686935002,
            "owned_by": "mistral",
        }),
    ];

    Ok(Json(serde_json::json!({
        "object": "list",
        "data": model_data,
    }))
    .into_response())
}

pub async fn chat_completions(
    State(state): State<AppState>,
    bearer: Option<TypedHeader<Authorization<Bearer>>>,
    WithRejection(Json(mut body), _): WithRejection<Json<ChatRequest>, Error>,
) -> crate::Result<Response> {
    state.valid_key(bearer)?;
    let mut token = None;
    for _ in 0..5 {
        match load_token(&state.client).await {
            Ok(new_token) => {
                token = Some(new_token);
                break;
            }
            Err(err) => {
                tracing::info!("retry load token: {:?}", err);
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        }
    }
    let token = token.ok_or_else(|| Error::BadRequest("cannot get token".to_string()))?;
    body.compress_messages();
    let (_, response) = send_request(&state.client, token, &body).await?;
    Ok(response)
}

async fn send_request(
    client: &Client,
    hash: String,
    body: &ChatRequest,
) -> Result<(String, Response)> {
    // dbg!(&hash);
    let resp = client
        .post("https://duck.ai/duckchat/v1/chat")
        .header(header::ACCEPT, "text/event-stream")
        .header(header::ORIGIN, ORIGIN_API)
        .header(header::REFERER, ORIGIN_API)
        .header("x-vqd-hash-1", hash)
        .json(&body)
        .send()
        .await?;

    let hash = resp
        .headers()
        .get("x-vqd-hash-1")
        .and_then(|header| header.to_str().ok())
        .ok_or_else(|| MissingHeader)?
        .to_owned();

    let response = ChatProcess::builder()
        .resp(resp)
        .stream(body.stream)
        .model(body.model.clone())
        .build()
        .into_response()
        .await?;

    Ok((hash, response))
}

async fn load_token(client: &Client) -> Result<String> {
    let resp = client
        .get("https://duck.ai/duckchat/v1/status")
        .header(header::REFERER, ORIGIN_API)
        .header("x-vqd-accept", "1")
        .send()
        .await?
        .error_for_status()?;

    let hash = resp
        .headers()
        .get("x-vqd-hash-1")
        .and_then(|header| header.to_str().ok())
        .ok_or_else(|| crate::Error::MissingHeader)?
        .to_owned();

    let request_hash = gen_request_hash(&hash)?;

    Ok(request_hash)
}
