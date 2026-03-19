use actix_web::{get, post, web, HttpResponse};
use diesel::prelude::*;

use crate::auth::middleware::AdminUser;
use crate::db::Pool;
use crate::errors::AppError;
use crate::models::user::{CreateUserRequest, NewUser, User, UserRole};
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

/// Seed an initial admin user (called at startup if no users exist)
pub fn seed_admin_if_empty(
    conn: &mut crate::db::DbConnection,
    admin_email: &str,
    password: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::schema::users::dsl::*;

    let count: i64 = users.count().get_result(conn)?;
    if count == 0 {
        let hash = bcrypt::hash(password, bcrypt::DEFAULT_COST)?;
        let new_admin = NewUser {
            name: "Administrator".to_string(),
            email: admin_email.to_string(),
            password_hash: hash,
            role: UserRole::Admin,
        };
        diesel::insert_into(users).values(&new_admin).execute(conn)?;
        log::info!("Seeded initial admin user: {}", admin_email);
    }
    Ok(())
}
