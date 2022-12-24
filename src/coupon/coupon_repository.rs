use super::model::{Coupon, CouponInsert, CouponUpdate};
use sqlx::{MySqlPool, query, query_as};
use sqlx::types::chrono::{NaiveDateTime};


pub async fn insert(coupon: CouponInsert, pool: &MySqlPool) -> Result<u64, sqlx::Error> {
    let result = query!(
        r#"
            INSERT INTO coupon 
            (code, discount, active, max_usage_count, expiration_date) 
            VALUES 
            (?, ?, ?, ?, ?)
        "#,
        coupon.code,
        coupon.discount,
        coupon.active,
        coupon.max_usage_count,
        coupon.expiration_date,
    )
    .execute(pool)
    .await
    .map_err(|error| {
        tracing::error!("Failed to execute insert query: {:?}", error);
        error
    })?;
    return Ok(result.last_insert_id());
}

pub async fn update(coupon: CouponUpdate, pool: &MySqlPool) -> Result<(), sqlx::Error> {
    query!(
        r#"
            UPDATE coupon SET
            discount = ?,
            active = ?,
            max_usage_count = ?,
            expiration_date = ?
            WHERE id = ?
        "#,
        coupon.discount,
        coupon.active,
        coupon.max_usage_count,
        coupon.expiration_date,
        coupon.id
    )
    .execute(pool)
    .await
    .map_err(|error| {
        tracing::error!("Failed to execute update query: {:?}", error);
        error
    })?;

    return Ok(());
}


pub async fn get_all(pool: &MySqlPool) -> Result<Vec<Coupon>, sqlx::Error> {
    let coupons = query_as!(Coupon,
        r#"SELECT id
        , code
        , discount 
        , max_usage_count
        , active as `active: bool`
        , expiration_date as `expiration_date: NaiveDateTime`
        , date_created as `date_created: Option<NaiveDateTime>`
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
    let field_name: &str;
    let field_value: String;
    match field {
        Fields::Id(id) => {
            field_name = "id";
            field_value = id.to_string();
        },
        Fields::Code(code) => {
            field_name = "code";
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
        , active as `active: bool`
        , expiration_date as `expiration_date: NaiveDateTime`
        , date_created as `date_created: Option<NaiveDateTime>`
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
        , active as `active: bool`
        , expiration_date as `expiration_date: NaiveDateTime`
        , date_created as `date_created: Option<NaiveDateTime>`
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

pub async fn get_by_code(code: &String, pool: &MySqlPool) -> Result<Option<Coupon>, sqlx::Error> {
    let coupon = query_as!(Coupon, 
        r#"SELECT id
        , code
        , discount 
        , max_usage_count
        , active as `active: bool`
        , expiration_date as `expiration_date: NaiveDateTime`
        , date_created as `date_created: Option<NaiveDateTime>`
        , date_updated as `date_updated: NaiveDateTime`
        FROM coupon WHERE code = ?
        "#, code
    )
    .fetch_optional(pool)
    .await
    .map_err(|error| {
        tracing::error!("Failed to execute select query: {:?}", error);
        error
    })?;

    return Ok(coupon);
}

pub async fn delete_by_id(id: i32, pool: &MySqlPool) -> Result<(), sqlx::Error> {
    query!( 
        r#"DELETE FROM coupon
            WHERE id = ?
        "#, id
    )
    .execute(pool)
    .await
    .map_err(|error| {
        tracing::error!("Failed to execute delete query: {:?}", error);
        error
    })?;

    return Ok(());
}

pub async fn delete_by_code(code: &String, pool: &MySqlPool) -> Result<(), sqlx::Error> {
    query!( 
        r#"DELETE FROM coupon
            WHERE code = ?
        "#, code
    )
    .execute(pool)
    .await
    .map_err(|error| {
        tracing::error!("Failed to execute delete query: {:?}", error);
        error
    })?;

    return Ok(());
}