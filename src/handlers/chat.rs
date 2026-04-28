use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::models::Claims;
use crate::services::push::PushService;

#[derive(Debug, Serialize, sqlx::FromRow, ToSchema)]
pub struct ConversationResponse {
    pub id: Uuid,
    pub car_id: Uuid,
    pub renter_id: Uuid,
    pub host_id: Uuid,
    pub last_message_text: String,
    pub last_message_at: NaiveDateTime,
    pub renter_unread_count: i32,
    pub host_unread_count: i32,
    pub status: String,
    pub created_at: NaiveDateTime,
    // Joined fields
    pub other_user_name: String,
    pub other_user_role: String,
    pub car_name: String,
    pub car_photo: String,
}

#[derive(Debug, Serialize, sqlx::FromRow, ToSchema)]
pub struct MessageResponse {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub sender_id: Uuid,
    pub content: String,
    pub message_type: String,
    pub reply_to_id: Option<Uuid>,
    pub is_read: bool,
    pub created_at: NaiveDateTime,
    // Joined
    pub sender_name: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateConversationRequest {
    pub car_id: Uuid,
    /// The other user in the conversation. Can be the host (if caller is renter)
    /// or the renter (if caller is host).
    #[serde(alias = "host_id")]
    pub other_user_id: Uuid,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SendMessageRequest {
    pub content: String,
    pub message_type: String,
    pub reply_to_id: Option<Uuid>,
}

/// POST /api/chat/conversations — Get an existing conversation for (car, renter) or create one
#[utoipa::path(
    post,
    path = "/api/chat/conversations",
    tag = "Chat",
    security(("bearer_auth" = [])),
    request_body = CreateConversationRequest,
    responses(
        (status = 200, description = "Existing conversation", body = ConversationResponse),
        (status = 201, description = "Newly created conversation", body = ConversationResponse),
        (status = 400, description = "Cannot start a conversation with yourself"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Car not found"),
    ),
)]
pub async fn get_or_create_conversation(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    body: web::Json<CreateConversationRequest>,
) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>().cloned() {
        Some(c) => c,
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Unauthorized"}))
        }
    };

    let caller_id = claims.sub;
    let car_id = body.car_id;
    let other_user_id = body.other_user_id;

    // Prevent chatting with yourself
    if caller_id == other_user_id {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({"error": "You cannot start a conversation with yourself"}));
    }

    // Determine who is the host and who is the renter by checking car ownership
    let car_host = sqlx::query_scalar::<_, Uuid>("SELECT host_id FROM cars WHERE id = $1")
        .bind(car_id)
        .fetch_optional(pool.get_ref())
        .await;

    let car_host_id = match car_host {
        Ok(Some(id)) => id,
        Ok(None) => {
            return HttpResponse::NotFound().json(serde_json::json!({"error": "Car not found"}));
        }
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": e.to_string()}));
        }
    };

    // If the caller owns the car, they are the host; the other user is the renter.
    // Otherwise, the caller is the renter and the other user is the host.
    let (renter_id, host_id) = if caller_id == car_host_id {
        (other_user_id, caller_id)
    } else {
        (caller_id, other_user_id)
    };

    // Try to find existing conversation
    let existing = sqlx::query_as::<_, ConversationResponse>(
        r#"SELECT
            conv.id, conv.car_id, conv.renter_id, conv.host_id,
            conv.last_message_text, conv.last_message_at,
            conv.renter_unread_count, conv.host_unread_count,
            conv.status, conv.created_at,
            CASE
                WHEN conv.renter_id = $3 THEN host_u.full_name
                ELSE renter_u.full_name
            END AS other_user_name,
            CASE
                WHEN conv.renter_id = $3 THEN host_u.role::text
                ELSE renter_u.role::text
            END AS other_user_role,
            CONCAT(c.make, ' ', c.model, ' ', c.year) AS car_name,
            COALESCE(c.photos[1], '') AS car_photo
        FROM conversations conv
        JOIN users renter_u ON renter_u.id = conv.renter_id
        JOIN users host_u ON host_u.id = conv.host_id
        JOIN cars c ON c.id = conv.car_id
        WHERE conv.car_id = $1 AND conv.renter_id = $2"#,
    )
    .bind(car_id)
    .bind(renter_id)
    .bind(caller_id)
    .fetch_optional(pool.get_ref())
    .await;

    match existing {
        Ok(Some(conversation)) => {
            return HttpResponse::Ok().json(conversation);
        }
        Ok(None) => {}
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": e.to_string()}));
        }
    }

    // Create new conversation
    let conv_id = Uuid::new_v4();
    let insert_result = sqlx::query(
        r#"INSERT INTO conversations
            (id, car_id, renter_id, host_id, last_message_text, last_message_at,
             renter_unread_count, host_unread_count, status, created_at)
        VALUES ($1, $2, $3, $4, '', NOW(), 0, 0, 'active', NOW())"#,
    )
    .bind(conv_id)
    .bind(car_id)
    .bind(renter_id)
    .bind(host_id)
    .execute(pool.get_ref())
    .await;

    if let Err(e) = insert_result {
        return HttpResponse::InternalServerError()
            .json(serde_json::json!({"error": e.to_string()}));
    }

    // Fetch the newly created conversation with joined data
    let result = sqlx::query_as::<_, ConversationResponse>(
        r#"SELECT
            conv.id, conv.car_id, conv.renter_id, conv.host_id,
            conv.last_message_text, conv.last_message_at,
            conv.renter_unread_count, conv.host_unread_count,
            conv.status, conv.created_at,
            CASE
                WHEN conv.renter_id = $2 THEN host_u.full_name
                ELSE renter_u.full_name
            END AS other_user_name,
            CASE
                WHEN conv.renter_id = $2 THEN host_u.role::text
                ELSE renter_u.role::text
            END AS other_user_role,
            CONCAT(c.make, ' ', c.model, ' ', c.year) AS car_name,
            COALESCE(c.photos[1], '') AS car_photo
        FROM conversations conv
        JOIN users renter_u ON renter_u.id = conv.renter_id
        JOIN users host_u ON host_u.id = conv.host_id
        JOIN cars c ON c.id = conv.car_id
        WHERE conv.id = $1"#,
    )
    .bind(conv_id)
    .bind(caller_id)
    .fetch_one(pool.get_ref())
    .await;

    match result {
        Ok(conversation) => HttpResponse::Created().json(conversation),
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        }
    }
}

/// GET /api/chat/conversations — List all conversations the user participates in
#[utoipa::path(
    get,
    path = "/api/chat/conversations",
    tag = "Chat",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Conversations sorted by last_message_at desc", body = Vec<ConversationResponse>),
        (status = 401, description = "Unauthorized"),
    ),
)]
pub async fn get_conversations(req: HttpRequest, pool: web::Data<PgPool>) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>().cloned() {
        Some(c) => c,
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Unauthorized"}))
        }
    };

    let user_id = claims.sub;

    let result = sqlx::query_as::<_, ConversationResponse>(
        r#"SELECT
            conv.id, conv.car_id, conv.renter_id, conv.host_id,
            conv.last_message_text, conv.last_message_at,
            conv.renter_unread_count, conv.host_unread_count,
            conv.status, conv.created_at,
            CASE
                WHEN conv.renter_id = $1 THEN host_u.full_name
                ELSE renter_u.full_name
            END AS other_user_name,
            CASE
                WHEN conv.renter_id = $1 THEN host_u.role::text
                ELSE renter_u.role::text
            END AS other_user_role,
            CONCAT(c.make, ' ', c.model, ' ', c.year) AS car_name,
            COALESCE(c.photos[1], '') AS car_photo
        FROM conversations conv
        JOIN users renter_u ON renter_u.id = conv.renter_id
        JOIN users host_u ON host_u.id = conv.host_id
        JOIN cars c ON c.id = conv.car_id
        WHERE conv.renter_id = $1 OR conv.host_id = $1
        ORDER BY conv.last_message_at DESC"#,
    )
    .bind(user_id)
    .fetch_all(pool.get_ref())
    .await;

    match result {
        Ok(conversations) => HttpResponse::Ok().json(conversations),
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        }
    }
}

/// GET /api/chat/conversations/{id}/messages — Fetch messages and mark them read
#[utoipa::path(
    get,
    path = "/api/chat/conversations/{id}/messages",
    tag = "Chat",
    security(("bearer_auth" = [])),
    params(("id" = Uuid, Path, description = "Conversation ID")),
    responses(
        (status = 200, description = "Messages in chronological order", body = Vec<MessageResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Not a participant in this conversation"),
    ),
)]
pub async fn get_messages(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>().cloned() {
        Some(c) => c,
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Unauthorized"}))
        }
    };

    let conversation_id = path.into_inner();
    let user_id = claims.sub;

    // Verify user is a participant
    let is_participant = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM conversations WHERE id = $1 AND (renter_id = $2 OR host_id = $2))",
    )
    .bind(conversation_id)
    .bind(user_id)
    .fetch_one(pool.get_ref())
    .await
    .unwrap_or(false);

    if !is_participant {
        return HttpResponse::Forbidden()
            .json(serde_json::json!({"error": "Not a participant in this conversation"}));
    }

    // Mark messages as read by resetting unread count for current user
    let _ = sqlx::query(
        r#"UPDATE conversations SET
            renter_unread_count = CASE WHEN renter_id = $2 THEN 0 ELSE renter_unread_count END,
            host_unread_count = CASE WHEN host_id = $2 THEN 0 ELSE host_unread_count END
        WHERE id = $1"#,
    )
    .bind(conversation_id)
    .bind(user_id)
    .execute(pool.get_ref())
    .await;

    // Also mark individual messages as read
    let _ = sqlx::query(
        "UPDATE messages SET is_read = true WHERE conversation_id = $1 AND sender_id != $2 AND is_read = false",
    )
    .bind(conversation_id)
    .bind(user_id)
    .execute(pool.get_ref())
    .await;

    let result = sqlx::query_as::<_, MessageResponse>(
        r#"SELECT
            m.id, m.conversation_id, m.sender_id, m.content,
            m.message_type, m.reply_to_id, m.is_read, m.created_at,
            u.full_name AS sender_name
        FROM messages m
        JOIN users u ON u.id = m.sender_id
        WHERE m.conversation_id = $1
        ORDER BY m.created_at ASC"#,
    )
    .bind(conversation_id)
    .fetch_all(pool.get_ref())
    .await;

    match result {
        Ok(messages) => HttpResponse::Ok().json(messages),
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        }
    }
}

/// POST /api/chat/conversations/{id}/messages — Send a message; pushes to the recipient
#[utoipa::path(
    post,
    path = "/api/chat/conversations/{id}/messages",
    tag = "Chat",
    security(("bearer_auth" = [])),
    params(("id" = Uuid, Path, description = "Conversation ID")),
    request_body = SendMessageRequest,
    responses(
        (status = 201, description = "Created message", body = MessageResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Not a participant in this conversation"),
    ),
)]
pub async fn send_message(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    push: web::Data<Option<PushService>>,
    path: web::Path<Uuid>,
    body: web::Json<SendMessageRequest>,
) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>().cloned() {
        Some(c) => c,
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Unauthorized"}))
        }
    };

    let conversation_id = path.into_inner();
    let user_id = claims.sub;

    // Verify user is a participant
    let is_participant = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM conversations WHERE id = $1 AND (renter_id = $2 OR host_id = $2))",
    )
    .bind(conversation_id)
    .bind(user_id)
    .fetch_one(pool.get_ref())
    .await
    .unwrap_or(false);

    if !is_participant {
        return HttpResponse::Forbidden()
            .json(serde_json::json!({"error": "Not a participant in this conversation"}));
    }

    let message_id = Uuid::new_v4();

    // Insert the message
    let insert_result = sqlx::query(
        r#"INSERT INTO messages
            (id, conversation_id, sender_id, content, message_type, reply_to_id, is_read, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, false, NOW())"#,
    )
    .bind(message_id)
    .bind(conversation_id)
    .bind(user_id)
    .bind(&body.content)
    .bind(&body.message_type)
    .bind(body.reply_to_id)
    .execute(pool.get_ref())
    .await;

    if let Err(e) = insert_result {
        return HttpResponse::InternalServerError()
            .json(serde_json::json!({"error": e.to_string()}));
    }

    // Update conversation: last message text, timestamp, and increment OTHER user's unread count
    let _ = sqlx::query(
        r#"UPDATE conversations SET
            last_message_text = $2,
            last_message_at = NOW(),
            renter_unread_count = CASE WHEN renter_id != $3 THEN renter_unread_count + 1 ELSE renter_unread_count END,
            host_unread_count = CASE WHEN host_id != $3 THEN host_unread_count + 1 ELSE host_unread_count END
        WHERE id = $1"#,
    )
    .bind(conversation_id)
    .bind(&body.content)
    .bind(user_id)
    .execute(pool.get_ref())
    .await;

    // Fetch and return the new message
    let result = sqlx::query_as::<_, MessageResponse>(
        r#"SELECT
            m.id, m.conversation_id, m.sender_id, m.content,
            m.message_type, m.reply_to_id, m.is_read, m.created_at,
            u.full_name AS sender_name
        FROM messages m
        JOIN users u ON u.id = m.sender_id
        WHERE m.id = $1"#,
    )
    .bind(message_id)
    .fetch_one(pool.get_ref())
    .await;

    match result {
        Ok(message) => {
            // Push notification to the recipient (the OTHER participant)
            if let Some(push) = push.get_ref().clone() {
                let recipient_id = sqlx::query_scalar::<_, Uuid>(
                    r#"SELECT CASE WHEN renter_id = $1 THEN host_id ELSE renter_id END
                       FROM conversations WHERE id = $2"#,
                )
                .bind(user_id)
                .bind(conversation_id)
                .fetch_optional(pool.get_ref())
                .await
                .ok()
                .flatten();

                if let Some(recipient_id) = recipient_id {
                    let pool = pool.get_ref().clone();
                    let title = message.sender_name.clone();
                    let body_text = body.content.clone();
                    let payload = serde_json::json!({
                        "type": "chat_message",
                        "conversation_id": conversation_id.to_string(),
                        "message_id": message_id.to_string(),
                    });
                    tokio::spawn(async move {
                        push.send_to_user(&pool, recipient_id, &title, &body_text, payload).await;
                    });
                }
            }
            HttpResponse::Created().json(message)
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        }
    }
}

/// DELETE /api/chat/conversations/{id} - Delete a conversation and its messages
#[utoipa::path(
    delete,
    path = "/api/chat/conversations/{id}",
    tag = "Chat",
    security(("bearer_auth" = [])),
    params(("id" = Uuid, Path, description = "Conversation ID")),
    responses(
        (status = 200, description = "Conversation deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Not a participant in this conversation"),
    ),
)]
pub async fn delete_conversation(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>().cloned() {
        Some(c) => c,
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Unauthorized"}))
        }
    };

    let conversation_id = path.into_inner();
    let user_id = claims.sub;

    // Verify user is a participant
    let is_participant = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM conversations WHERE id = $1 AND (renter_id = $2 OR host_id = $2))",
    )
    .bind(conversation_id)
    .bind(user_id)
    .fetch_one(pool.get_ref())
    .await
    .unwrap_or(false);

    if !is_participant {
        return HttpResponse::Forbidden()
            .json(serde_json::json!({"error": "Not a participant in this conversation"}));
    }

    // Delete messages first (FK constraint), then conversation
    let _ = sqlx::query("DELETE FROM messages WHERE conversation_id = $1")
        .bind(conversation_id)
        .execute(pool.get_ref())
        .await;

    let result = sqlx::query("DELETE FROM conversations WHERE id = $1")
        .bind(conversation_id)
        .execute(pool.get_ref())
        .await;

    match result {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({"message": "Conversation deleted"})),
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        }
    }
}

/// POST /api/chat/conversations/{id}/read — Reset the caller's unread count for a conversation
#[utoipa::path(
    post,
    path = "/api/chat/conversations/{id}/read",
    tag = "Chat",
    security(("bearer_auth" = [])),
    params(("id" = Uuid, Path, description = "Conversation ID")),
    responses(
        (status = 200, description = "Marked as read"),
        (status = 401, description = "Unauthorized"),
    ),
)]
pub async fn mark_read(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>().cloned() {
        Some(c) => c,
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Unauthorized"}))
        }
    };

    let conversation_id = path.into_inner();
    let user_id = claims.sub;

    // Reset unread count for the current user
    let _ = sqlx::query(
        r#"UPDATE conversations SET
            renter_unread_count = CASE WHEN renter_id = $2 THEN 0 ELSE renter_unread_count END,
            host_unread_count = CASE WHEN host_id = $2 THEN 0 ELSE host_unread_count END
        WHERE id = $1 AND (renter_id = $2 OR host_id = $2)"#,
    )
    .bind(conversation_id)
    .bind(user_id)
    .execute(pool.get_ref())
    .await;

    HttpResponse::Ok().json(serde_json::json!({"message": "Marked as read"}))
}
