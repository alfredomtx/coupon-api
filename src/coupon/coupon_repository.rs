use sqlx::{MySqlPool, query, query_as};
use sqlx::types::chrono::{NaiveDateTime};

use super::model::{Coupon, CouponInsert};

pub async fn insert(coupon: CouponInsert, pool: &MySqlPool) -> Result<u64, sqlx::Error> {
    let result = query!(
        r#"
            INSERT INTO coupon (code, discount, max_usage_count)
            VALUES (?, ?, ?)
        "#,
        coupon.code, coupon.discount, coupon.max_usage_count
    )
    .execute(pool)
    .await
    .map_err(|error| {
        tracing::error!("Failed to execute insert query: {:?}", error);
        error
    })?;
    return Ok(result.last_insert_id());
}

pub async fn get_all(pool: &MySqlPool) -> Result<Vec<Coupon>, sqlx::Error> {
    let coupons = query_as!(Coupon,
        r#"SELECT id
        , code
        , discount 
        , max_usage_count
        , expiration_date as `expiration_date: NaiveDateTime`
        , date_created as `date_created: NaiveDateTime`
        , date_updated as `date_updated: NaiveDateTime`
        FROM coupon"#)
    .fetch_all(pool)
    .await
    .map_err(|error| {
        tracing::error!("Failed to execute select query: {:?}", error);
        error
    })?;

   return Ok(coupons);
}

pub enum Fields {
    Id(i32),
    Code(String),
    None,
}

pub async fn get_by_field(field: Fields, pool: &MySqlPool) -> Result<Option<Coupon>, sqlx::Error> {
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

    let coupon = query_as!(Coupon, 
        r#"SELECT id
        , code
        , discount 
        , max_usage_count
        , expiration_date as `expiration_date: chrono::NaiveDateTime`
        , date_created as `date_created: NaiveDateTime`
        , date_updated as `date_updated: NaiveDateTime`
        FROM coupon WHERE ? = ?
        "#, field_name, field_value
    )
    .fetch_optional(pool)
    .await
    .map_err(|error| {
        tracing::error!("Failed to execute select query: {:?}", error);
        error
    })?;

    return Ok(coupon);

}

pub async fn get_by_id(id: i32, pool: &MySqlPool) -> Result<Option<Coupon>, sqlx::Error> {
    let coupon = query_as!(Coupon, 
        r#"SELECT id
        , code
        , discount 
        , max_usage_count
        , expiration_date as `expiration_date: chrono::NaiveDateTime`
        , date_created as `date_created: NaiveDateTime`
        , date_updated as `date_updated: NaiveDateTime`
        FROM coupon WHERE id = ?
        "#, id
    )
    .fetch_optional(pool)
    .await
    .map_err(|error| {
        tracing::error!("Failed to execute select query: {:?}", error);
        error
    })?;

    return Ok(coupon);
}

