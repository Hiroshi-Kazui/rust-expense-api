use actix_web::{get, post, web, HttpResponse};
use diesel::prelude::*;

use crate::auth::middleware::AdminUser;
use crate::db::Pool;
use crate::errors::AppError;
use crate::models::user::{CreateUserRequest, NewUser, User};
use crate::schema::users;

#[post("/users")]
pub async fn create_user(
    pool: web::Data<Pool>,
    _admin: AdminUser,
    body: web::Json<CreateUserRequest>,
) -> Result<HttpResponse, AppError> {
    let mut conn = pool.get().map_err(|e| AppError::Internal(e.to_string()))?;

    let password_hash = bcrypt::hash(&body.password, bcrypt::DEFAULT_COST)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let new_user = NewUser {
        name: body.name.clone(),
        email: body.email.clone(),
        password_hash,
        role: body.role.clone(),
    };

    let user: User = diesel::insert_into(users::table)
        .values(&new_user)
        .returning(User::as_returning())
        .get_result(&mut conn)
        .map_err(|e| match e {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                _,
            ) => AppError::BadRequest("Email already exists".into()),
            other => AppError::from(other),
        })?;

    Ok(HttpResponse::Created().json(user))
}

#[get("/users")]
pub async fn list_users(
    pool: web::Data<Pool>,
    _admin: AdminUser,
) -> Result<HttpResponse, AppError> {
    let mut conn = pool.get().map_err(|e| AppError::Internal(e.to_string()))?;

    let user_list: Vec<User> = users::table
        .select(User::as_select())
        .order(users::created_at.asc())
        .load(&mut conn)
        .map_err(AppError::from)?;

    Ok(HttpResponse::Ok().json(user_list))
}

