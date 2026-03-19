use actix_multipart::Multipart;
use actix_web::{delete, get, patch, post, web, HttpResponse};
use diesel::prelude::*;
use uuid::Uuid;

use crate::auth::middleware::{AdminUser, AuthenticatedUser};
use crate::config::Config;
use crate::db::Pool;
use crate::errors::AppError;
use crate::models::expense::{
    BulkApproveRequest, Expense, ExpenseStatus, NewExpense, UpdateExpense, UpdateExpenseRequest,
    UpdateExpenseStatus, UpdateStatusRequest,
};
use crate::models::user::UserRole;
use crate::schema::expenses;

// ─── POST /expenses ───────────────────────────────────────────────────────────

#[post("/expenses")]
pub async fn create_expense(
    pool: web::Data<Pool>,
    config: web::Data<Config>,
    auth: AuthenticatedUser,
    multipart: Multipart,
) -> Result<HttpResponse, AppError> {
    // Parse multipart: text fields + optional receipt file
    let (fields, receipt_path) =
        parse_expense_multipart(multipart, &config.upload_dir).await?;

    let category_id: Uuid = fields
        .get("category_id")
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| AppError::BadRequest("category_id is required".into()))?;

    let amount: i32 = fields
        .get("amount")
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| AppError::BadRequest("amount is required and must be a number".into()))?;

    if amount <= 0 {
        return Err(AppError::BadRequest("amount must be a positive integer".into()));
    }

    let purpose = fields
        .get("purpose")
        .cloned()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| AppError::BadRequest("purpose is required".into()))?;

    let occurred_at = fields
        .get("occurred_at")
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| AppError::BadRequest("occurred_at is required (YYYY-MM-DD)".into()))?;

    let note = fields.get("note").cloned().filter(|s| !s.is_empty());

    let new_expense = NewExpense {
        user_id: auth.0.sub,
        category_id,
        amount,
        purpose,
        occurred_at,
        note,
        receipt_file: receipt_path.clone(),
    };

    let mut conn = pool.get().map_err(|e| AppError::Internal(e.to_string()))?;

    let expense: Expense = diesel::insert_into(expenses::table)
        .values(&new_expense)
        .returning(Expense::as_returning())
        .get_result(&mut conn)
        .map_err(|e| {
            // DB 挿入失敗時はアップロード済みファイルを削除
            if let Some(ref path) = receipt_path {
                let full = std::path::PathBuf::from(&config.upload_dir).join(path);
                std::fs::remove_file(full).ok();
            }
            AppError::from(e)
        })?;

    Ok(HttpResponse::Created().json(expense))
}

// ─── GET /expenses ────────────────────────────────────────────────────────────
// admin: all, user: own only

#[get("/expenses")]
pub async fn list_expenses(
    pool: web::Data<Pool>,
    auth: AuthenticatedUser,
) -> Result<HttpResponse, AppError> {
    let mut conn = pool.get().map_err(|e| AppError::Internal(e.to_string()))?;

    let list: Vec<Expense> = if auth.0.role == UserRole::Admin {
        expenses::table
            .select(Expense::as_select())
            .order(expenses::created_at.desc())
            .load(&mut conn)
            .map_err(AppError::from)?
    } else {
        expenses::table
            .filter(expenses::user_id.eq(auth.0.sub))
            .select(Expense::as_select())
            .order(expenses::created_at.desc())
            .load(&mut conn)
            .map_err(AppError::from)?
    };

    Ok(HttpResponse::Ok().json(list))
}

// ─── GET /expenses/{id} ───────────────────────────────────────────────────────

#[get("/expenses/{id}")]
pub async fn get_expense(
    pool: web::Data<Pool>,
    auth: AuthenticatedUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let expense_id = path.into_inner();
    let mut conn = pool.get().map_err(|e| AppError::Internal(e.to_string()))?;

    let expense: Expense = expenses::table
        .find(expense_id)
        .select(Expense::as_select())
        .first(&mut conn)
        .map_err(AppError::from)?;

    if auth.0.role != UserRole::Admin && expense.user_id != auth.0.sub {
        return Err(AppError::Forbidden("Not your expense".into()));
    }

    Ok(HttpResponse::Ok().json(expense))
}

// ─── PATCH /expenses/{id} ─────────────────────────────────────────────────────

#[patch("/expenses/{id}")]
pub async fn update_expense(
    pool: web::Data<Pool>,
    auth: AuthenticatedUser,
    path: web::Path<Uuid>,
    body: web::Json<UpdateExpenseRequest>,
) -> Result<HttpResponse, AppError> {
    let expense_id = path.into_inner();
    let mut conn = pool.get().map_err(|e| AppError::Internal(e.to_string()))?;

    let expense: Expense = expenses::table
        .find(expense_id)
        .select(Expense::as_select())
        .first(&mut conn)
        .map_err(AppError::from)?;

    if expense.user_id != auth.0.sub {
        return Err(AppError::Forbidden("Not your expense".into()));
    }
    if expense.status != ExpenseStatus::Pending {
        return Err(AppError::BadRequest(
            "Only pending expenses can be edited".into(),
        ));
    }

    let changes = UpdateExpense {
        category_id: body.category_id,
        amount: body.amount,
        purpose: body.purpose.clone(),
        occurred_at: body.occurred_at,
        note: body.note.clone(),
        receipt_file: None,
    };

    let updated: Expense = diesel::update(expenses::table.find(expense_id))
        .set(&changes)
        .returning(Expense::as_returning())
        .get_result(&mut conn)
        .map_err(AppError::from)?;

    Ok(HttpResponse::Ok().json(updated))
}

// ─── DELETE /expenses/{id} ────────────────────────────────────────────────────

#[delete("/expenses/{id}")]
pub async fn delete_expense(
    pool: web::Data<Pool>,
    auth: AuthenticatedUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let expense_id = path.into_inner();
    let mut conn = pool.get().map_err(|e| AppError::Internal(e.to_string()))?;

    let expense: Expense = expenses::table
        .find(expense_id)
        .select(Expense::as_select())
        .first(&mut conn)
        .map_err(AppError::from)?;

    if expense.user_id != auth.0.sub {
        return Err(AppError::Forbidden("Not your expense".into()));
    }
    if expense.status != ExpenseStatus::Pending {
        return Err(AppError::BadRequest(
            "Only pending expenses can be deleted".into(),
        ));
    }

    diesel::delete(expenses::table.find(expense_id))
        .execute(&mut conn)
        .map_err(AppError::from)?;

    Ok(HttpResponse::NoContent().finish())
}

// ─── PATCH /expenses/{id}/status  (admin) ────────────────────────────────────

#[patch("/expenses/{id}/status")]
pub async fn update_expense_status(
    pool: web::Data<Pool>,
    _admin: AdminUser,
    path: web::Path<Uuid>,
    body: web::Json<UpdateStatusRequest>,
) -> Result<HttpResponse, AppError> {
    let expense_id = path.into_inner();
    let mut conn = pool.get().map_err(|e| AppError::Internal(e.to_string()))?;

    let changes = UpdateExpenseStatus {
        status: body.status.clone(),
    };

    let updated: Expense = diesel::update(expenses::table.find(expense_id))
        .set(&changes)
        .returning(Expense::as_returning())
        .get_result(&mut conn)
        .map_err(AppError::from)?;

    Ok(HttpResponse::Ok().json(updated))
}

// ─── POST /expenses/bulk-approve  (admin) ─────────────────────────────────────

#[post("/expenses/bulk-approve")]
pub async fn bulk_approve(
    pool: web::Data<Pool>,
    _admin: AdminUser,
    body: web::Json<BulkApproveRequest>,
) -> Result<HttpResponse, AppError> {
    if body.ids.is_empty() {
        return Err(AppError::BadRequest("ids must not be empty".into()));
    }

    let mut conn = pool.get().map_err(|e| AppError::Internal(e.to_string()))?;

    let count = diesel::update(
        expenses::table
            .filter(expenses::id.eq_any(&body.ids))
            .filter(expenses::status.eq(ExpenseStatus::Pending)),
    )
    .set(expenses::status.eq(ExpenseStatus::Approved))
    .execute(&mut conn)
    .map_err(AppError::from)?;

    Ok(HttpResponse::Ok().json(serde_json::json!({ "approved": count })))
}

// ─── Multipart helper ─────────────────────────────────────────────────────────

async fn parse_expense_multipart(
    multipart: Multipart,
    upload_dir: &str,
) -> Result<(std::collections::HashMap<String, String>, Option<String>), AppError> {
    use futures_util::StreamExt;
    use std::collections::HashMap;

    let mut fields: HashMap<String, String> = HashMap::new();
    let mut receipt_path: Option<String> = None;
    let mut mp = multipart;

    while let Some(item) = mp.next().await {
        let mut field = item.map_err(|e| AppError::BadRequest(e.to_string()))?;
        let cd = field.content_disposition();
        let name = cd.get_name().unwrap_or("").to_string();

        if name == "receipt" {
            let content_type = field
                .content_type()
                .map(|m| m.to_string())
                .unwrap_or_default();

            const ALLOWED: &[&str] = &["image/jpeg", "image/png", "application/pdf"];
            if !ALLOWED.contains(&content_type.as_str()) {
                return Err(AppError::BadRequest(format!(
                    "Unsupported file type: {}",
                    content_type
                )));
            }

            let ext = match content_type.as_str() {
                "image/jpeg" => "jpg",
                "image/png" => "png",
                "application/pdf" => "pdf",
                _ => "bin",
            };

            let file_name = format!("{}.{}", uuid::Uuid::new_v4(), ext);
            let file_path = std::path::PathBuf::from(upload_dir).join(&file_name);

            tokio::fs::create_dir_all(upload_dir)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?;

            let mut file = tokio::fs::File::create(&file_path)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?;

            let mut total = 0usize;
            use tokio::io::AsyncWriteExt;
            while let Some(chunk) = field.next().await {
                let data = chunk.map_err(|e| AppError::BadRequest(e.to_string()))?;
                file.write_all(&data)
                    .await
                    .map_err(|e| AppError::Internal(e.to_string()))?;
                total += data.len();
                if total > 10 * 1024 * 1024 {
                    drop(file);
                    let _ = tokio::fs::remove_file(&file_path).await;
                    return Err(AppError::BadRequest("File too large (max 10MB)".into()));
                }
            }

            receipt_path = Some(file_name);
        } else {
            // Text field
            let mut value = Vec::new();
            while let Some(chunk) = field.next().await {
                let data = chunk.map_err(|e| AppError::BadRequest(e.to_string()))?;
                value.extend_from_slice(&data);
            }
            let text = String::from_utf8(value)
                .map_err(|_| AppError::BadRequest(format!("Field {} is not valid UTF-8", name)))?;
            fields.insert(name, text);
        }
    }

    Ok((fields, receipt_path))
}
