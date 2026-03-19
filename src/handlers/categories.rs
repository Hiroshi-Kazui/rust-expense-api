use actix_web::{get, post, web, HttpResponse};
use diesel::prelude::*;

use crate::auth::middleware::AdminUser;
use crate::db::Pool;
use crate::errors::AppError;
use crate::models::category::{Category, CreateCategoryRequest, NewCategory};
use crate::schema::categories;

#[post("/categories")]
pub async fn create_category(
    pool: web::Data<Pool>,
    _admin: AdminUser,
    body: web::Json<CreateCategoryRequest>,
) -> Result<HttpResponse, AppError> {
    let mut conn = pool.get().map_err(|e| AppError::Internal(e.to_string()))?;

    if body.name.trim().is_empty() {
        return Err(AppError::BadRequest("Category name cannot be empty".into()));
    }

    let new_category = NewCategory {
        name: body.name.trim().to_string(),
    };

    let category: Category = diesel::insert_into(categories::table)
        .values(&new_category)
        .returning(Category::as_returning())
        .get_result(&mut conn)
        .map_err(AppError::from)?;

    Ok(HttpResponse::Created().json(category))
}

#[get("/categories")]
pub async fn list_categories(
    pool: web::Data<Pool>,
    _admin: AdminUser,
) -> Result<HttpResponse, AppError> {
    let mut conn = pool.get().map_err(|e| AppError::Internal(e.to_string()))?;

    let list: Vec<Category> = categories::table
        .select(Category::as_select())
        .order(categories::name.asc())
        .load(&mut conn)
        .map_err(AppError::from)?;

    Ok(HttpResponse::Ok().json(list))
}
