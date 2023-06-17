use std::sync::Arc;

use eyre::Result;
use sqlx::MySqlPool;

use crate::{
    config::crypto::CryptoService,
    models::user::{NewUser, UpdateProfile, User},
};

#[derive(Clone)]
pub struct UserRepository {
    pool: Arc<MySqlPool>,
}

impl UserRepository {
    pub fn new(pool: Arc<MySqlPool>) -> Self {
        Self { pool }
    }

    pub async fn find_by_email(&self, email: String) -> Result<User> {
        let user = sqlx::query_as!(
            User,
            "SELECT id, email, fullName AS full_name, password, image FROM User WHERE email = ?",
            email
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(user)
    }

    pub async fn create(&self, new_user: NewUser, crypto_service: &CryptoService) -> Result<User> {
        let password_hash: String = crypto_service.hash_password(new_user.password).await?;

        let mut tx = self.pool.begin().await?;

        sqlx::query!(
            r#"
                UPDATE User
                SET password = ?
                WHERE email = ?
            "#,
            password_hash,
            new_user.email,
        )
        .execute(&mut tx)
        .await?;

        let user = sqlx::query_as!(
            User,
            "SELECT id, email, fullName AS full_name, password, image FROM User WHERE email = ?",
            new_user.email
        )
        .fetch_one(&mut tx)
        .await?;

        tx.commit().await?;

        Ok(user)
    }

    pub async fn update(&self, update_profile: UpdateProfile, email: String) -> Result<User> {
        let dynamic_update_statement: Vec<String> = update_profile
            .as_hashmap()
            .into_iter()
            .map(|kv| -> String { format!("{} = nullif('{}', ''), ", kv.0, kv.1) })
            .collect();
        let dynamic_update_statement = dynamic_update_statement.join(" ");
        let dynamic_update_statement: String = dynamic_update_statement
            .chars()
            .take(dynamic_update_statement.len() - 2)
            .collect();

        println!("{}\n", dynamic_update_statement);

        let mut tx = self.pool.begin().await?;

        sqlx::query(
            format!(
                r#"
                    UPDATE User
                    SET {}
                    WHERE email = ?
                "#,
                dynamic_update_statement
            )
            .as_str(),
        )
        .bind(email.clone())
        .execute(&mut tx)
        .await?;

        let user = sqlx::query_as!(
            User,
            "SELECT id, email, fullName AS full_name, password, image FROM User WHERE email = ?",
            email
        )
        .fetch_one(&mut tx)
        .await?;

        tx.commit().await?;

        Ok(user)
    }

    pub async fn delete(&self, email: String) -> Result<User> {
        let mut tx = self.pool.begin().await?;

        let user = sqlx::query_as!(
            User,
            "SELECT id, email, fullName AS full_name, password, image FROM User WHERE email = ?",
            email
        )
        .fetch_one(&mut tx)
        .await?;

        sqlx::query!(
            r#"
                DELETE FROM User 
                WHERE email = ? 
            "#,
            email
        )
        .execute(&mut tx)
        .await?;

        tx.commit().await?;

        Ok(user)
    }
}
