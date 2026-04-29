use actix::prelude::*;
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use uuid::Uuid;

use crate::models::Claims;

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(10);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(30);

// ─── Messages between actors ────────────────────────────────────────────────

/// Message sent to a specific user's WebSocket sessions
#[derive(Clone, Message, Serialize, Deserialize, Debug)]
#[rtype(result = "()")]
pub struct WsMessage {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub payload: serde_json::Value,
}

/// A user connected
#[derive(Message)]
#[rtype(result = "()")]
struct Connect {
    user_id: Uuid,
    addr: Addr<ChatWsSession>,
}

/// A user disconnected
#[derive(Message)]
#[rtype(result = "()")]
struct Disconnect {
    user_id: Uuid,
    addr: Addr<ChatWsSession>,
}

/// Send a message to a specific user
#[derive(Message)]
#[rtype(result = "()")]
pub struct SendToUser {
    pub user_id: Uuid,
    pub message: WsMessage,
}

/// Mark which conversation a session is currently viewing — lets the server
/// suppress push notifications for messages the user is already looking at.
#[derive(Message)]
#[rtype(result = "()")]
struct SetActiveConversation {
    user_id: Uuid,
    addr: Addr<ChatWsSession>,
    conversation_id: Option<Uuid>,
}

/// Query whether a user has the given conversation open in any session.
#[derive(Message)]
#[rtype(result = "bool")]
pub struct IsConversationActive {
    pub user_id: Uuid,
    pub conversation_id: Uuid,
}

// ─── Connection Manager (Actor) ─────────────────────────────────────────────

/// Per-session state: the recipient address + which conversation (if any)
/// they currently have open in the foreground.
struct SessionEntry {
    addr: Addr<ChatWsSession>,
    active_conversation: Option<Uuid>,
}

pub struct WsManager {
    sessions: HashMap<Uuid, Vec<SessionEntry>>,
}

impl WsManager {
    pub fn new() -> Self {
        WsManager {
            sessions: HashMap::new(),
        }
    }
}

impl Actor for WsManager {
    type Context = Context<Self>;
}

impl Handler<Connect> for WsManager {
    type Result = ();

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) {
        log::info!("WS: User {} connected", msg.user_id);
        self.sessions.entry(msg.user_id).or_default().push(SessionEntry {
            addr: msg.addr,
            active_conversation: None,
        });
    }
}

impl Handler<Disconnect> for WsManager {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        if let Some(sessions) = self.sessions.get_mut(&msg.user_id) {
            sessions.retain(|s| s.addr != msg.addr);
            if sessions.is_empty() {
                self.sessions.remove(&msg.user_id);
            }
        }
        log::info!("WS: User {} disconnected", msg.user_id);
    }
}

impl Handler<SendToUser> for WsManager {
    type Result = ();

    fn handle(&mut self, msg: SendToUser, _: &mut Context<Self>) {
        if let Some(sessions) = self.sessions.get(&msg.user_id) {
            for entry in sessions {
                entry.addr.do_send(WsMessage {
                    msg_type: msg.message.msg_type.clone(),
                    payload: msg.message.payload.clone(),
                });
            }
        }
    }
}

impl Handler<SetActiveConversation> for WsManager {
    type Result = ();

    fn handle(&mut self, msg: SetActiveConversation, _: &mut Context<Self>) {
        if let Some(sessions) = self.sessions.get_mut(&msg.user_id) {
            for entry in sessions.iter_mut() {
                if entry.addr == msg.addr {
                    entry.active_conversation = msg.conversation_id;
                }
            }
        }
    }
}

impl Handler<IsConversationActive> for WsManager {
    type Result = bool;

    fn handle(&mut self, msg: IsConversationActive, _: &mut Context<Self>) -> bool {
        self.sessions
            .get(&msg.user_id)
            .map(|sessions| {
                sessions
                    .iter()
                    .any(|s| s.active_conversation == Some(msg.conversation_id))
            })
            .unwrap_or(false)
    }
}

// ─── WebSocket Session (per-connection actor) ───────────────────────────────

pub struct ChatWsSession {
    user_id: Uuid,
    hb: Instant,
    manager: Addr<WsManager>,
    pool: web::Data<PgPool>,
    push: web::Data<Option<crate::services::push::PushService>>,
}

impl ChatWsSession {
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                log::warn!("WS: heartbeat timeout for user {}", act.user_id);
                ctx.stop();
                return;
            }
            ctx.ping(b"");
        });
    }
}

impl Actor for ChatWsSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);
        self.manager.do_send(Connect {
            user_id: self.user_id,
            addr: ctx.address(),
        });
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        self.manager.do_send(Disconnect {
            user_id: self.user_id,
            addr: ctx.address(),
        });
    }
}

/// Handle incoming WsMessage from manager → forward to client
impl Handler<WsMessage> for ChatWsSession {
    type Result = ();

    fn handle(&mut self, msg: WsMessage, ctx: &mut Self::Context) {
        if let Ok(text) = serde_json::to_string(&msg) {
            ctx.text(text);
        }
    }
}

/// Handle raw WebSocket frames from client
impl actix::StreamHandler<Result<ws::Message, ws::ProtocolError>> for ChatWsSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        let msg = match msg {
            Ok(m) => m,
            Err(_) => {
                ctx.stop();
                return;
            }
        };

        match msg {
            ws::Message::Ping(m) => {
                self.hb = Instant::now();
                ctx.pong(&m);
            }
            ws::Message::Pong(_) => {
                self.hb = Instant::now();
            }
            ws::Message::Text(text) => {
                self.hb = Instant::now();
                let user_id = self.user_id;
                let manager = self.manager.clone();
                let pool = self.pool.clone();

                // Parse incoming JSON message
                if let Ok(incoming) = serde_json::from_str::<serde_json::Value>(&text) {
                    let msg_type = incoming["type"].as_str().unwrap_or("").to_string();

                    match msg_type.as_str() {
                        "chat_message" => {
                            let conversation_id = incoming["conversation_id"]
                                .as_str()
                                .unwrap_or("")
                                .to_string();
                            let content = incoming["content"].as_str().unwrap_or("").to_string();
                            let message_type = incoming["message_type"]
                                .as_str()
                                .unwrap_or("text")
                                .to_string();

                            // Save to DB and broadcast
                            actix::spawn(async move {
                                if let Ok(conv_id) = Uuid::parse_str(&conversation_id) {
                                    let msg_id = Uuid::new_v4();
                                    let result = sqlx::query(
                                        r#"INSERT INTO messages (id, conversation_id, sender_id, content, message_type, is_read, created_at)
                                        VALUES ($1, $2, $3, $4, $5, false, NOW())"#,
                                    )
                                    .bind(msg_id)
                                    .bind(conv_id)
                                    .bind(user_id)
                                    .bind(&content)
                                    .bind(&message_type)
                                    .execute(pool.get_ref())
                                    .await;

                                    if result.is_ok() {
                                        // Update conversation timestamp
                                        let _ = sqlx::query(
                                            "UPDATE conversations SET last_message = $1, last_message_at = NOW(), updated_at = NOW() WHERE id = $2",
                                        )
                                        .bind(&content)
                                        .bind(conv_id)
                                        .execute(pool.get_ref())
                                        .await;

                                        // Get the other participant
                                        let participants = sqlx::query_as::<_, (Uuid, Uuid)>(
                                            "SELECT renter_id, host_id FROM conversations WHERE id = $1",
                                        )
                                        .bind(conv_id)
                                        .fetch_optional(pool.get_ref())
                                        .await;

                                        if let Ok(Some((renter_id, host_id))) = participants {
                                            let recipient = if user_id == renter_id {
                                                host_id
                                            } else {
                                                renter_id
                                            };

                                            let payload = serde_json::json!({
                                                "id": msg_id.to_string(),
                                                "conversation_id": conversation_id,
                                                "sender_id": user_id.to_string(),
                                                "content": content,
                                                "message_type": message_type,
                                                "created_at": chrono::Utc::now().to_rfc3339(),
                                            });

                                            // Send to recipient
                                            manager.do_send(SendToUser {
                                                user_id: recipient,
                                                message: WsMessage {
                                                    msg_type: "new_message".to_string(),
                                                    payload: payload.clone(),
                                                },
                                            });

                                            // Echo back to sender (confirmation)
                                            manager.do_send(SendToUser {
                                                user_id,
                                                message: WsMessage {
                                                    msg_type: "message_sent".to_string(),
                                                    payload,
                                                },
                                            });
                                        }
                                    }
                                }
                            });
                        }
                        "typing" => {
                            let conversation_id = incoming["conversation_id"]
                                .as_str()
                                .unwrap_or("")
                                .to_string();
                            let is_typing = incoming["is_typing"].as_bool().unwrap_or(false);

                            actix::spawn(async move {
                                if let Ok(conv_id) = Uuid::parse_str(&conversation_id) {
                                    let participants = sqlx::query_as::<_, (Uuid, Uuid)>(
                                        "SELECT renter_id, host_id FROM conversations WHERE id = $1",
                                    )
                                    .bind(conv_id)
                                    .fetch_optional(pool.get_ref())
                                    .await;

                                    if let Ok(Some((renter_id, host_id))) = participants {
                                        let recipient = if user_id == renter_id {
                                            host_id
                                        } else {
                                            renter_id
                                        };
                                        manager.do_send(SendToUser {
                                            user_id: recipient,
                                            message: WsMessage {
                                                msg_type: "typing".to_string(),
                                                payload: serde_json::json!({
                                                    "conversation_id": conversation_id,
                                                    "user_id": user_id.to_string(),
                                                    "is_typing": is_typing,
                                                }),
                                            },
                                        });
                                    }
                                }
                            });
                        }
                        "view_conversation" => {
                            // Client says: I have this conversation open in the
                            // foreground. Suppress push notifications for it
                            // until they tell us otherwise.
                            let conversation_id = incoming["conversation_id"]
                                .as_str()
                                .and_then(|s| Uuid::parse_str(s).ok());
                            manager.do_send(SetActiveConversation {
                                user_id,
                                addr: ctx.address(),
                                conversation_id,
                            });
                        }
                        "call_offer" | "call_answer" | "ice_candidate" | "call_hangup"
                        | "call_reject" => {
                            let target_id =
                                incoming["target_id"].as_str().unwrap_or("").to_string();
                            let push = self.push.clone();

                            actix::spawn(async move {
                                if let Ok(target) = Uuid::parse_str(&target_id) {
                                    let mut payload = incoming.clone();
                                    payload["sender_id"] = serde_json::json!(user_id.to_string());

                                    manager.do_send(SendToUser {
                                        user_id: target,
                                        message: WsMessage {
                                            msg_type: msg_type.clone(),
                                            payload: payload.clone(),
                                        },
                                    });

                                    // Wake the device with a push so the
                                    // recipient sees the call even if the WS
                                    // session is suspended (iOS background).
                                    if msg_type == "call_offer" {
                                        if let Some(push_svc) = push.get_ref().clone() {
                                            // Fetch caller name for the push title
                                            let caller_name = sqlx::query_scalar::<_, String>(
                                                "SELECT full_name FROM users WHERE id = $1",
                                            )
                                            .bind(user_id)
                                            .fetch_one(pool.get_ref())
                                            .await
                                            .unwrap_or_else(|_| "Someone".to_string());

                                            let push_payload = serde_json::json!({
                                                "type": "incoming_call",
                                                "sender_id": user_id.to_string(),
                                                "conversation_id": incoming["conversation_id"],
                                            });
                                            let pool = pool.get_ref().clone();
                                            tokio::spawn(async move {
                                                push_svc
                                                    .send_to_user(
                                                        &pool,
                                                        target,
                                                        &caller_name,
                                                        "Incoming call…",
                                                        push_payload,
                                                    )
                                                    .await;
                                            });
                                        }
                                    }
                                }
                            });
                        }
                        _ => {}
                    }
                }
            }
            ws::Message::Close(reason) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => {}
        }
    }
}

// ─── HTTP upgrade handler ───────────────────────────────────────────────────

pub async fn ws_connect(
    req: HttpRequest,
    stream: web::Payload,
    pool: web::Data<PgPool>,
    manager: web::Data<Addr<WsManager>>,
    push: web::Data<Option<crate::services::push::PushService>>,
) -> Result<HttpResponse, Error> {
    // Extract token from query string: ?token=JWT
    let query = req.query_string();
    let token = query
        .split('&')
        .find(|p| p.starts_with("token="))
        .and_then(|p| p.strip_prefix("token="))
        .unwrap_or("");

    if token.is_empty() {
        return Ok(HttpResponse::Unauthorized().json(serde_json::json!({"error": "Missing token"})));
    }

    // Verify JWT
    let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_default();
    let token_data = jsonwebtoken::decode::<Claims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(jwt_secret.as_bytes()),
        &jsonwebtoken::Validation::default(),
    );

    let claims = match token_data {
        Ok(data) => data.claims,
        Err(_) => {
            return Ok(
                HttpResponse::Unauthorized().json(serde_json::json!({"error": "Invalid token"}))
            );
        }
    };

    log::info!("WS: Upgrading connection for user {}", claims.sub);

    ws::start(
        ChatWsSession {
            user_id: claims.sub,
            hb: Instant::now(),
            manager: manager.get_ref().clone(),
            pool,
            push,
        },
        &req,
        stream,
    )
}
