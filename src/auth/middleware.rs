use actix_web::{web, FromRequest, HttpRequest};
use futures::future::{ready, Ready};

use crate::auth::jwt::{verify_token, Claims};
use crate::config::Config;
use crate::errors::AppError;
use crate::models::user::UserRole;

pub struct AuthenticatedUser(pub Claims);

impl FromRequest for AuthenticatedUser {
    type Error = AppError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        let config = req.app_data::<web::Data<Config>>().cloned();

        let result = (|| {
            let config = config.ok_or_else(|| AppError::Internal("Config not found".into()))?;

            let auth_header = req
                .headers()
                .get("Authorization")
                .ok_or_else(|| AppError::Unauthorized("Authorization header missing".into()))?
                .to_str()
                .map_err(|_| AppError::Unauthorized("Invalid Authorization header".into()))?;

            let token = auth_header
                .strip_prefix("Bearer ")
                .ok_or_else(|| AppError::Unauthorized("Bearer token required".into()))?;

            verify_token(token, &config.jwt_secret)
        })();

        ready(result.map(AuthenticatedUser))
    }
}

pub struct AdminUser(pub Claims);

impl FromRequest for AdminUser {
    type Error = AppError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, payload: &mut actix_web::dev::Payload) -> Self::Future {
        let result = AuthenticatedUser::from_request(req, payload)
            .into_inner()
            .and_then(|auth| {
                if auth.0.role == UserRole::Admin {
                    Ok(AdminUser(auth.0))
                } else {
                    Err(AppError::Forbidden("Admin role required".into()))
                }
            });
        ready(result)
    }
}
