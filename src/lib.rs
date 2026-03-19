pub mod auth;
pub mod config;
pub mod db;
pub mod errors;
pub mod handlers;
pub mod models;
pub mod schema;
pub mod seeder;
pub mod upload;

use diesel_migrations::{embed_migrations, EmbeddedMigrations};
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");
