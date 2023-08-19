type Pool = sqlx::PgPool;

#[derive(Debug, sqlx::FromRow)]
pub struct User {
    pub id: i32,
    pub user_id: String,
    pub username: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl User {
    // pub fn new_default(id: String) -> Self {
    //     User {
    //         id,
    //         username: "default_username".to_string(),
    //         created_at: chrono::Utc::now(),
    //         updated_at: chrono::Utc::now(),
    //     }
    // }

    pub async fn insert_new(user_id: &str, username: &str, pool: &Pool) -> sqlx::Result<u64> {
        let res = sqlx::query!(
            "insert into users (user_id, username) values ($1, $2);",
            user_id,
            username
        )
        .execute(pool)
        .await?;
        Ok(res.rows_affected())
    }

    pub async fn get_from_user_id(user_id: &str, pool: &Pool) -> sqlx::Result<Self> {
        sqlx::query_as!(User, "select * from users where user_id = $1", user_id)
            .fetch_one(pool)
            .await
    }

    pub async fn get_id_from_user_id(user_id: &str, pool: &Pool) -> sqlx::Result<i32> {
        let record = sqlx::query!("select id from users where user_id = $1", user_id)
            .fetch_one(pool)
            .await?;
        Ok(record.id)
    }

    pub async fn update_username_from_user_id(
        user_id: &str,
        new_username: &str,
        pool: &Pool,
    ) -> sqlx::Result<u64> {
        let res = sqlx::query!(
            "update users set username = $1 where user_id = $2",
            new_username,
            user_id
        )
        .execute(pool)
        .await?;

        Ok(res.rows_affected())
    }
}
