use std::net::SocketAddr;

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        ConnectInfo, Query, State,
    },
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::Utc;
use futures::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    events::Event,
    model::{auth::UserStatus, user::User},
    repo::user::UserRepo,
    utils::extractors::Session,
    AppState,
};

#[derive(Deserialize)]
pub struct UserStatusQuery {
    id: Uuid,
}

#[derive(Serialize)]
struct UserStatusMessage {
    #[serde(rename = "onlineStatus")]
    online_status: UserStatus,
    #[serde(rename = "lastActiveAt")]
    last_active_at: Option<String>,
}

pub async fn user_status(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Query(query): Query<UserStatusQuery>,
    State(state): State<AppState>,
) -> Result<Response, Response> {
    let mut conn = state.db.acquire().await.unwrap();
    let user = UserRepo::get_by_id(query.id, &mut conn)
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, "User not found").into_response())?;
    Ok(ws
        .on_upgrade(move |socket| handle_user_status_socket(socket, addr, state, user))
        .into_response())
}
async fn handle_user_status_socket(
    socket: WebSocket,
    addr: SocketAddr,
    state: AppState,
    user: User,
) {
    let (mut socket_tx, mut socket_rx) = socket.split();
    let chan = state.event_channel;
    let mut event_receiver = chan.subscribe();

    if let Err(e) = socket_tx
        .send(Message::Text(
            serde_json::to_string(&UserStatusMessage {
                online_status: user.online_status,
                last_active_at: user.last_active_at.map(|t| t.to_rfc3339()),
            })
            .unwrap(),
        ))
        .await
    {
        tracing::error!("Error sending message to {addr}: {e}");
    }

    let mut event_rx_task = tokio::spawn(async move {
        while let Ok(event) = event_receiver.recv().await {
            match event {
                Event::UserStatusUpdate {
                    user_id: event_user_id,
                    new_status,
                } => {
                    if event_user_id == user.id {
                        let send_message = UserStatusMessage {
                            online_status: new_status,
                            last_active_at: None,
                        };
                        if let Err(e) = socket_tx
                            .send(Message::Text(serde_json::to_string(&send_message).unwrap()))
                            .await
                        {
                            tracing::error!("Error sending message to {addr}: {e}");
                        }
                    }
                }
            }
        }
    });

    let mut socket_rx_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = socket_rx.next().await {
            if let Message::Close(_) = msg {
                break;
            }
        }
    });

    tokio::select! {
        rv_a = (&mut socket_rx_task) => {
            match rv_a {
                Ok(_) => println!("messages received from {addr}"),
                Err(e) => println!("Error receiving messages from {addr}: {e}"),
            }
            event_rx_task.abort();
        }
        rv_b = (&mut event_rx_task) => {
            match rv_b {
                Ok(_) => println!("messages received from {addr}"),
                Err(e) => println!("Error receiving messages from {addr}: {e}"),
            }
            socket_rx_task.abort();
        }
    }

    tracing::info!("Disconnected: {addr}");
}

pub async fn user_me_status(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<AppState>,
    Session(user): Session<User>,
) -> impl IntoResponse {
    if let Some(user) = user {
        // Ignore the result, because this errors if there are no receivers in the channel
        let _ = state.event_channel.publish(Event::UserStatusUpdate {
            user_id: user.id,
            new_status: UserStatus::Online,
        });
        // TODO: update in database
        ws.on_upgrade(move |socket| handle_user_me_status_socket(socket, addr, state, user))
    } else {
        (StatusCode::UNAUTHORIZED, "Unauthorized").into_response()
    }
}

#[derive(Deserialize, Serialize)]
struct UserMeStatusMessage {
    #[serde(rename = "onlineStatus")]
    online_status: UserStatus,
}

async fn handle_user_me_status_socket(
    socket: WebSocket,
    addr: SocketAddr,
    state: AppState,
    user: User,
) {
    let (mut socket_tx, mut socket_rx) = socket.split();
    let chan = state.event_channel;
    let mut conn = state.db.acquire().await.unwrap();

    socket_tx
        .send(Message::Text(serde_json::to_string(&user.id).unwrap()))
        .await
        .unwrap();

    let socket_rx_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = socket_rx.next().await {
            match msg {
                Message::Text(text) => {
                    let event: UserMeStatusMessage = match serde_json::from_str(&text) {
                        Ok(event) => event,
                        Err(e) => {
                            socket_tx.send(Message::Text(e.to_string())).await.unwrap();
                            continue;
                        }
                    };
                    let last_active_at = if matches!(
                        event.online_status,
                        UserStatus::Online | UserStatus::Away | UserStatus::DoNotDisturb
                    ) || (event.online_status == UserStatus::Offline
                        && matches!(
                            user.online_status,
                            UserStatus::Online | UserStatus::Away | UserStatus::DoNotDisturb
                        )) {
                        Some(Utc::now())
                    } else {
                        None
                    };
                    UserRepo::update_status(
                        user.id,
                        event.online_status.clone(),
                        last_active_at,
                        &mut conn,
                    )
                    .await
                    .unwrap(); // TODO: handle error

                    let send_message = Event::UserStatusUpdate {
                        user_id: user.id,
                        new_status: event.online_status,
                    };
                    tracing::info!("Text message from {addr}: {text}");
                    // Ignore the result, because it errors if there are no receivers in the channel
                    let _ = chan.publish(send_message);
                }
                Message::Close(_) => {
                    // Ignore the result, because this errors if there are no receivers in the channel
                    let _ = chan.publish(Event::UserStatusUpdate {
                        user_id: user.id,
                        new_status: UserStatus::Offline,
                    });
                    UserRepo::update_status(
                        user.id,
                        UserStatus::Offline,
                        Some(Utc::now()),
                        &mut conn,
                    )
                    .await
                    .unwrap(); // TODO: handle error
                    tracing::info!("Disconnected from me-user-status: {addr}");
                    break;
                }
                _ => (),
            }
        }
    });
    socket_rx_task.await.unwrap();
}
