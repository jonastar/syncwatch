use std::{
    net::SocketAddr,
    str::FromStr,
    sync::{Arc, RwLock},
    time::Duration,
};

use axum::{
    extract::{
        ws::{Message, WebSocket},
        WebSocketUpgrade,
    },
    handler::Handler,
    headers,
    http::Uri,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Extension, Json, Router, TypedHeader,
};

use clap::Parser;
use events::UpdateEvent;
use player::PlayerStateHandle;
use serde::Deserialize;
use tokio::time::timeout;

use crate::{cors::CorsLayer, player::PlayerState};

mod config;
mod cors;
mod events;
mod player;

#[tokio::main]
async fn main() {
    let loaded_config = config::Args::parse();

    println!(
        "Starting syncwatch backend on {}",
        loaded_config.listen_addr
    );

    let player = Arc::new(RwLock::new(PlayerState::new()));

    let router_b = Router::new()
        .route("/ws", get(ws_handler))
        .layer(Extension(player.clone()));

    let app = Router::new()
        .route("/change_media", post(handle_change_media))
        .route("/seek", post(handle_seek))
        .route("/pause", post(handle_pause))
        .route("/unpause", post(handle_unpause))
        .fallback(fallback.into_service())
        .layer(Extension(Arc::new(loaded_config.clone())))
        .layer(Extension(player.clone()))
        .layer(CorsLayer)
        .merge(router_b);

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = SocketAddr::from_str(&loaded_config.listen_addr).unwrap();
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

fn check_admin(headers: &HeaderMap, config: &config::Args) -> Result<(), StatusCode> {
    let auth_header = headers
        .get("Authorization")
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if auth_header.to_str().map_err(|_| StatusCode::UNAUTHORIZED)? == config.admin_pw {
        Ok(())
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

#[derive(Deserialize)]
struct ChangeMediaBody {
    new_url: String,
}

async fn handle_change_media(
    headers: HeaderMap,
    config: Extension<Arc<config::Args>>,
    player: Extension<PlayerStateHandle>,
    query: Json<ChangeMediaBody>,
) -> Result<StatusCode, StatusCode> {
    check_admin(&headers, &config)?;

    let mut w = player.write().unwrap();
    w.change_media(query.new_url.clone());

    Ok(StatusCode::OK)
}

async fn handle_pause(
    headers: HeaderMap,
    config: Extension<Arc<config::Args>>,
    player: Extension<PlayerStateHandle>,
) -> Result<StatusCode, StatusCode> {
    check_admin(&headers, &config)?;

    let mut w = player.write().unwrap();
    w.pause();

    Ok(StatusCode::OK)
}

async fn handle_unpause(
    headers: HeaderMap,
    config: Extension<Arc<config::Args>>,
    player: Extension<PlayerStateHandle>,
) -> Result<StatusCode, StatusCode> {
    check_admin(&headers, &config)?;

    let mut w = player.write().unwrap();
    w.unpause();

    Ok(StatusCode::OK)
}

#[derive(Deserialize)]
struct SeekBody {
    new_ts_milliseconds: u32,
}

async fn handle_seek(
    headers: HeaderMap,
    config: Extension<Arc<config::Args>>,
    player: Extension<PlayerStateHandle>,
    query: Json<SeekBody>,
) -> Result<StatusCode, StatusCode> {
    check_admin(&headers, &config)?;

    let mut w = player.write().unwrap();
    w.seek(Duration::from_millis(query.new_ts_milliseconds as u64));

    Ok(StatusCode::OK)
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    player: Extension<PlayerStateHandle>,
) -> impl IntoResponse {
    if let Some(TypedHeader(user_agent)) = user_agent {
        println!("`{}` connected", user_agent.as_str());
    }

    ws.on_upgrade(move |ws| handle_socket(ws, player.0))
}

async fn handle_socket(mut socket: WebSocket, player: PlayerStateHandle) {
    // send initial status update
    let mut sub = { player.read().unwrap().subscribe() };

    // send initial player state
    // the "ready" event
    {
        let serialized = {
            let last = player.read().unwrap().current_state();
            serde_json::to_string(&UpdateEvent {
                ts_millis: last.ts.as_millis() as u64,
                state: last.state,
                media_url: last.media_url,
            })
            .unwrap()
        };

        if let Err(err) = socket.send(Message::Text(serialized)).await {
            println!("client disconnected {}", err);
            return;
        }
    }

    loop {
        // because people might close our connection if we don't send anything for a while, we continuously
        // send status update

        let update = match timeout(Duration::from_secs(5), sub.changed()).await {
            Err(_) => {
                // timeout, send status update
                player.read().unwrap().current_state()
            }

            Ok(Err(err)) => {
                // failed fetching stuff?
                eprintln!("failed watching for latest state: {err}");
                return;
            }

            Ok(Ok(_)) => {
                // OK!
                sub.borrow_and_update().clone()
            }
        };

        let serialized = {
            serde_json::to_string(&UpdateEvent {
                ts_millis: update.ts.as_millis() as u64,
                state: update.state,
                media_url: update.media_url,
            })
            .unwrap()
        };

        if let Err(err) = socket.send(Message::Text(serialized)).await {
            println!("client disconnected {}", err);
            return;
        }
    }
}

async fn fallback(uri: Uri) -> (StatusCode, String) {
    (StatusCode::NOT_FOUND, format!("No route for {}", uri))
}
