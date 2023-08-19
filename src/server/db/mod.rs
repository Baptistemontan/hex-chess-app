use actix_web::web;
use sqlx::PgPool;

pub mod user;

pub async fn create_db_pool() -> sqlx::Result<web::Data<PgPool>> {
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set.");

    let pool = PgPool::connect(&db_url).await?;
    Ok(web::Data::new(pool))
}
