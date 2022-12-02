use sqlx::{MySqlPool, query, query_as};
use sqlx::types::chrono::{NaiveDateTime};

use super::model::{Cupom, CupomInsert};

pub async fn insert(cupom: CupomInsert, pool: &MySqlPool) -> Result<u64, sqlx::Error> {
    let result = query!(
        r#"
            INSERT INTO cupom (code, discount, max_usage_count)
            VALUES (?, ?, ?)
        "#,
        cupom.code, cupom.discount, cupom.max_usage_count
    )
    .execute(pool)
    .await
    .map_err(|error| {
        tracing::error!("Failed to execute insert query: {:?}", error);
        error
    })?;
    return Ok(result.last_insert_id());
}

pub async fn get_all(pool: &MySqlPool) -> Result<Vec<Cupom>, sqlx::Error> {
    let cupoms = query_as!(Cupom,
        r#"SELECT id
        , code
        , discount 
        , max_usage_count
        , expiration_date as `expiration_date: NaiveDateTime`
        , date_created as `date_created: NaiveDateTime`
        , date_updated as `date_updated: NaiveDateTime`
        FROM cupom"#)
    .fetch_all(pool)
    .await
    .map_err(|error| {
        tracing::error!("Failed to execute select query: {:?}", error);
        error
    })?;

   return Ok(cupoms);
}

pub enum Fields {
    Id(i32),
    Code(String),
    None,
}

pub async fn get_by_field(field: Fields, pool: &MySqlPool) -> Result<Option<Cupom>, sqlx::Error> {
    let field_name: String;
    let field_value: String;
    match field {
        Fields::Id(id) => {
            field_name = "id".to_string();
            field_value = id.to_string();
        },
        Fields::Code(code) => {
            field_name = "code".to_string();
            field_value = code.to_string();
        }
        Fields::None => {
            return Ok(None);
            
        }
    }

    let cupom = query_as!(Cupom, 
        r#"SELECT id
        , code
        , discount 
        , max_usage_count
        , expiration_date as `expiration_date: chrono::NaiveDateTime`
        , date_created as `date_created: NaiveDateTime`
        , date_updated as `date_updated: NaiveDateTime`
        FROM cupom WHERE ? = ?
        "#, field_name, field_value
    )
    .fetch_optional(pool)
    .await
    .map_err(|error| {
        tracing::error!("Failed to execute select query: {:?}", error);
        error
    })?;

    return Ok(cupom);

}

pub async fn get_by_id(id: i32, pool: &MySqlPool) -> Result<Option<Cupom>, sqlx::Error> {
    let cupom = query_as!(Cupom, 
        r#"SELECT id
        , code
        , discount 
        , max_usage_count
        , expiration_date as `expiration_date: chrono::NaiveDateTime`
        , date_created as `date_created: NaiveDateTime`
        , date_updated as `date_updated: NaiveDateTime`
        FROM cupom WHERE id = ?
        "#, id
    )
    .fetch_optional(pool)
    .await
    .map_err(|error| {
        tracing::error!("Failed to execute select query: {:?}", error);
        error
    })?;

    return Ok(cupom);
}

