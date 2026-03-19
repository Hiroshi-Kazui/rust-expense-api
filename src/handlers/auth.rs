use actix_web::{post, web, HttpResponse};
use diesel::prelude::*;

use crate::config::Config;
use crate::db::Pool;
use crate::errors::AppError;
use crate::models::user::{LoginRequest, LoginResponse, User};
use crate::schema::users;

#[post("/auth/login")]
pub async fn login(
    pool: web::Data<Pool>,
    config: web::Data<Config>,
    body: web::Json<LoginRequest>,
) -> Result<HttpResponse, AppError> {
    let mut conn = pool.get().map_err(|e| AppError::Internal(e.to_string()))?;

    let user: User = users::table
        .filter(users::email.eq(&body.email))
        .select(User::as_select())
        .first(&mut conn)
        .map_err(|_| AppError::Unauthorized("Invalid email or password".into()))?;

    let valid = bcrypt::verify(&body.password, &user.password_hash)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    if !valid {
        return Err(AppError::Unauthorized("Invalid email or password".into()));
    }

    let token = crate::auth::jwt::generate_token(
        user.id,
        user.role.clone(),
        &config.jwt_secret,
        config.jwt_expires_in_hours,
    )?;

    Ok(HttpResponse::Ok().json(LoginResponse { token, user }))
}
