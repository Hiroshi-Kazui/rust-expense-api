use chrono::NaiveDate;
use diesel::prelude::*;

use crate::db::DbConnection;
use crate::models::category::NewCategory;
use crate::models::expense::NewExpense;
use crate::models::user::{NewUser, UserRole};
use crate::schema::{categories, expenses, users};

/// Always runs at startup: admin user + default categories.
pub fn run(conn: &mut DbConnection) -> Result<(), Box<dyn std::error::Error>> {
    let admin_email =
        std::env::var("ADMIN_EMAIL").unwrap_or_else(|_| "admin@example.com".to_string());
    let admin_password = std::env::var("ADMIN_PASSWORD").unwrap_or_else(|_| {
        log::warn!("ADMIN_PASSWORD not set, using default 'changeme'. Change this immediately!");
        "changeme".to_string()
    });

    seed_admin(conn, &admin_email, &admin_password)?;
    seed_categories(conn)?;

    let demo = std::env::var("SEED_DEMO").unwrap_or_default();
    if demo == "true" || demo == "1" {
        seed_demo_data(conn)?;
    }

    Ok(())
}

// ─── Admin ───────────────────────────────────────────────────────────────────

fn seed_admin(
    conn: &mut DbConnection,
    email: &str,
    password: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let count: i64 = users::table.count().get_result(conn)?;
    if count > 0 {
        return Ok(());
    }

    let hash = bcrypt::hash(password, bcrypt::DEFAULT_COST)?;
    diesel::insert_into(users::table)
        .values(NewUser {
            name: "Administrator".to_string(),
            email: email.to_string(),
            password_hash: hash,
            role: UserRole::Admin,
        })
        .execute(conn)?;
    log::info!("Seeder: admin user created ({})", email);
    Ok(())
}

// ─── Categories ───────────────────────────────────────────────────────────────

fn seed_categories(conn: &mut DbConnection) -> Result<(), Box<dyn std::error::Error>> {
    let count: i64 = categories::table.count().get_result(conn)?;
    if count > 0 {
        return Ok(());
    }

    let default_categories = vec![
        "交通費",
        "宿泊費",
        "接待費",
        "会議費",
        "消耗品費",
        "通信費",
        "研修費",
        "その他",
    ];

    let rows: Vec<NewCategory> = default_categories
        .into_iter()
        .map(|name| NewCategory {
            name: name.to_string(),
        })
        .collect();

    diesel::insert_into(categories::table)
        .values(&rows)
        .execute(conn)?;
    log::info!("Seeder: {} categories created", rows.len());
    Ok(())
}

// ─── Demo data (SEED_DEMO=true) ───────────────────────────────────────────────

fn seed_demo_data(conn: &mut DbConnection) -> Result<(), Box<dyn std::error::Error>> {
    seed_demo_users(conn)?;
    seed_demo_expenses(conn)?;
    Ok(())
}

fn seed_demo_users(conn: &mut DbConnection) -> Result<(), Box<dyn std::error::Error>> {
    use crate::schema::users::dsl::*;

    let demo_users = vec![
        ("田中 太郎", "tanaka@example.com", "password123"),
        ("鈴木 花子", "suzuki@example.com", "password123"),
    ];

    for (user_name, user_email, pw) in demo_users {
        let exists: i64 = users
            .filter(email.eq(user_email))
            .count()
            .get_result(conn)?;
        if exists > 0 {
            continue;
        }
        let hash = bcrypt::hash(pw, bcrypt::DEFAULT_COST)?;
        diesel::insert_into(users)
            .values(NewUser {
                name: user_name.to_string(),
                email: user_email.to_string(),
                password_hash: hash,
                role: UserRole::User,
            })
            .execute(conn)?;
        log::info!("Seeder: demo user created ({})", user_email);
    }
    Ok(())
}

fn seed_demo_expenses(conn: &mut DbConnection) -> Result<(), Box<dyn std::error::Error>> {
    // Skip if expenses already exist
    let count: i64 = expenses::table.count().get_result(conn)?;
    if count > 0 {
        return Ok(());
    }

    // Resolve user and category IDs
    let tanaka_id: Option<uuid::Uuid> = users::table
        .filter(users::email.eq("tanaka@example.com"))
        .select(users::id)
        .first(conn)
        .optional()?;

    let suzuki_id: Option<uuid::Uuid> = users::table
        .filter(users::email.eq("suzuki@example.com"))
        .select(users::id)
        .first(conn)
        .optional()?;

    let cat_transport: Option<uuid::Uuid> = categories::table
        .filter(categories::name.eq("交通費"))
        .select(categories::id)
        .first(conn)
        .optional()?;

    let cat_accommodation: Option<uuid::Uuid> = categories::table
        .filter(categories::name.eq("宿泊費"))
        .select(categories::id)
        .first(conn)
        .optional()?;

    let cat_entertainment: Option<uuid::Uuid> = categories::table
        .filter(categories::name.eq("接待費"))
        .select(categories::id)
        .first(conn)
        .optional()?;

    let (Some(tanaka), Some(suzuki), Some(transport), Some(accommodation), Some(entertainment)) = (
        tanaka_id,
        suzuki_id,
        cat_transport,
        cat_accommodation,
        cat_entertainment,
    ) else {
        log::warn!("Seeder: skipping demo expenses (required users/categories not found)");
        return Ok(());
    };

    let rows = vec![
        NewExpense {
            user_id: tanaka,
            category_id: transport,
            amount: 3_500,
            purpose: "東京-大阪 新幹線代".to_string(),
            occurred_at: NaiveDate::from_ymd_opt(2026, 3, 1).unwrap(),
            note: None,
            receipt_file: None,
        },
        NewExpense {
            user_id: tanaka,
            category_id: accommodation,
            amount: 12_000,
            purpose: "大阪出張 ホテル代".to_string(),
            occurred_at: NaiveDate::from_ymd_opt(2026, 3, 2).unwrap(),
            note: Some("1泊".to_string()),
            receipt_file: None,
        },
        NewExpense {
            user_id: suzuki,
            category_id: entertainment,
            amount: 28_000,
            purpose: "クライアント接待 ディナー".to_string(),
            occurred_at: NaiveDate::from_ymd_opt(2026, 3, 10).unwrap(),
            note: Some("4名参加".to_string()),
            receipt_file: None,
        },
        NewExpense {
            user_id: suzuki,
            category_id: transport,
            amount: 840,
            purpose: "営業先訪問 電車代".to_string(),
            occurred_at: NaiveDate::from_ymd_opt(2026, 3, 15).unwrap(),
            note: None,
            receipt_file: None,
        },
    ];

    let inserted = rows.len();
    diesel::insert_into(expenses::table)
        .values(&rows)
        .execute(conn)?;
    log::info!("Seeder: {} demo expenses created", inserted);
    Ok(())
}
