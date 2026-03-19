use actix_web::{middleware::Logger, web, App, HttpServer};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

mod auth;
mod config;
mod db;
mod errors;
mod handlers;
mod models;
mod schema;
mod seeder;
mod upload;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    env_logger::init();

    let cfg = config::Config::from_env();
    let pool = db::create_pool(&cfg.database_url);

    // Run migrations
    {
        let mut conn = pool.get().expect("Failed to get DB connection for migrations");
        conn.run_pending_migrations(MIGRATIONS)
            .expect("Failed to run migrations");
        log::info!("Database migrations applied");
    }

    // Run seeders
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
        App::new()
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
            // Auth
            .service(handlers::auth::login)
            // Users (admin)
            .service(handlers::users::create_user)
            .service(handlers::users::list_users)
            // Categories (admin)
            .service(handlers::categories::create_category)
            .service(handlers::categories::list_categories)
            // Expenses — bulk-approve must come before /{id} routes
            .service(handlers::expenses::bulk_approve)
            .service(handlers::expenses::create_expense)
            .service(handlers::expenses::list_expenses)
            .service(handlers::expenses::update_expense)
            .service(handlers::expenses::delete_expense)
            .service(handlers::expenses::update_expense_status)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
