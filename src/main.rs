use rust_expense_api::{errors, handlers, seeder, MIGRATIONS};
use actix_cors::Cors;
use actix_web::{http, middleware::Logger, web, App, HttpServer};
use diesel_migrations::MigrationHarness;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    env_logger::init();

    let cfg = rust_expense_api::config::Config::from_env();
    let pool = rust_expense_api::db::create_pool(&cfg.database_url);

    {
        let mut conn = pool.get().expect("Failed to get DB connection for migrations");
        conn.run_pending_migrations(MIGRATIONS)
            .expect("Failed to run migrations");
        log::info!("Database migrations applied");
    }

    {
        let mut conn = pool.get().expect("Failed to get DB connection for seeding");
        if let Err(e) = seeder::run(&mut conn) {
            log::warn!("Seeder failed (non-fatal): {}", e);
        }
    }

    let pool_data = web::Data::new(pool);
    let cfg_data = web::Data::new(cfg);

    log::info!("Starting server at http://0.0.0.0:8080");

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000")
            .allowed_methods(vec!["GET", "POST", "PUT", "PATCH", "DELETE"])
            .allowed_headers(vec![http::header::AUTHORIZATION, http::header::CONTENT_TYPE])
            .max_age(3600);

        App::new()
            .wrap(cors)
            .wrap(Logger::default())
            .app_data(pool_data.clone())
            .app_data(cfg_data.clone())
            .app_data(
                web::JsonConfig::default()
                    .error_handler(|err, _| {
                        let response = errors::AppError::BadRequest(err.to_string());
                        actix_web::error::InternalError::from_response(
                            err,
                            actix_web::ResponseError::error_response(&response),
                        )
                        .into()
                    }),
            )
            .service(handlers::auth::login)
            .service(handlers::users::create_user)
            .service(handlers::users::list_users)
            .service(handlers::categories::create_category)
            .service(handlers::categories::list_categories)
            .service(handlers::expenses::bulk_approve)
            .service(handlers::expenses::create_expense)
            .service(handlers::expenses::list_expenses)
            .service(handlers::expenses::get_expense)
            .service(handlers::expenses::update_expense)
            .service(handlers::expenses::delete_expense)
            .service(handlers::expenses::update_expense_status)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
