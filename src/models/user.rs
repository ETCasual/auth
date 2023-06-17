use async_graphql::{InputObject, SimpleObject};
use chrono::{DateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};
use sqlx::mysql::MySqlTypeInfo;
use sqlx::MySql;
use std::collections::HashMap;

lazy_static! {
    static ref RE_HAS_ONE_ALPHABET: Regex = Regex::new(r"[A-Za-z]+").unwrap();
    static ref RE_HAS_ONE_NUMBER: Regex = Regex::new(r"[0-9]+").unwrap();
}

// #[derive(Debug, Deserialize, Serialize, sqlx::Type)]
// #[sqlx(type_name = "status")]
// pub enum Status {
//     #[sqlx(rename = "ZL_TL")]
//     ZLTL,
//     CGL,
//     OM,
//     NB,
//     NF,
// }

// #[derive(Debug, Deserialize, Serialize, sqlx::Type)]
// #[sqlx(type_name = "cluster")]
// pub enum Cluster {
//     Heart,
//     Strike,
//     Force,
//     Move,
//     Mind,
//     Voice,
// }

// #[derive(Debug, Deserialize, Serialize, sqlx::Type)]
// #[sqlx(type_name = "cgRole")]
// pub enum CGRole {
//     CGL,
//     FL,
//     Member,
// }

// #[derive(Debug, Deserialize, Serialize, sqlx::Type)]
// #[sqlx(type_name = "gender")]
// pub enum Gender {
//     Male,
//     Female,
// }

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow, SimpleObject)]
pub struct User {
    pub id: i32,
    // pub created_at: DateTime<Utc>,
    // pub updated_at: DateTime<Utc>,
    // pub status: Vec<u8>,
    // pub age: i32,
    // pub ic_no: Vec<u8>,
    // pub phone_number: Option<Vec<u8>>,
    pub image: Option<Vec<u8>>,
    // pub cluster: Option<Vec<u8>>,
    pub email: Vec<u8>,
    pub full_name: Vec<u8>,
    // pub nick_name: Option<Vec<u8>>,
    // pub cg_role: Vec<u8>,
    #[serde(skip)]
    #[graphql(skip)]
    pub password: Option<Vec<u8>>,
    // pub address_id: Vec<u8>,
    // pub cell_id: Vec<u8>,
    // pub gender: Vec<u8>,
}

impl sqlx::Type<MySql> for User {
    fn type_info() -> MySqlTypeInfo {
        String::type_info()
    }

    fn compatible(ty: &MySqlTypeInfo) -> bool {
        String::compatible(ty)
    }
}

#[derive(Debug, Deserialize, Serialize, SimpleObject)]
pub struct UserWithAuth {
    pub access_token: String,
    pub expired_at: DateTime<Utc>,
    pub user: User,
}

#[derive(Clone, Debug, Deserialize, Validate, InputObject)]
pub struct NewUser {
    #[validate(email(message = "must be a valid email"))]
    pub email: String,
    #[validate(length(min = 8, message = " Password must have at least 8 characters"))]
    #[validate(regex(
        path = "RE_HAS_ONE_ALPHABET",
        message = "Password must have at least one alphabet"
    ))]
    #[validate(regex(
        path = "RE_HAS_ONE_NUMBER",
        message = "Password must have at least one number"
    ))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate, InputObject)]
pub struct UpdateProfile {
    #[validate(length(min = 3, message = "must be at least 3 characters"))]
    pub full_name: Option<String>,
    #[validate(url(message = "image must be a valid URL"))]
    pub image: Option<String>,
}

impl UpdateProfile {
    pub fn as_hashmap(&self) -> HashMap<String, String> {
        let mut hashmap = HashMap::new();

        match self.full_name.as_ref() {
            Some(x) => {
                hashmap.insert("full_name".to_string(), x.clone());
            }
            None => {
                hashmap.insert("full_name".to_string(), "".to_string());
            }
        };
        match self.image.as_ref() {
            Some(x) => {
                hashmap.insert("image".to_string(), x.clone());
            }
            None => {
                hashmap.insert("image".to_string(), "".to_string());
            }
        };

        hashmap
    }
}

#[derive(Clone, Debug, Deserialize, Validate, InputObject)]
pub struct SignInWithEmail {
    #[validate(email(message = "must be a valid email"))]
    pub email: String,
    #[validate(length(min = 8, message = " Password must have at least 8 characters"))]
    #[validate(regex(
        path = "RE_HAS_ONE_ALPHABET",
        message = "Password must have at least one alphabet"
    ))]
    #[validate(regex(
        path = "RE_HAS_ONE_NUMBER",
        message = "Password must have at least one number"
    ))]
    pub password: String,
}
