use actix_web::{
    web, get, post, HttpResponse,
};
use super::model::{CupomRequest, CupomResponse, CupomError};
use super::cupom_service::*;
use sqlx::{
    query,
    MySqlPool,
};
use anyhow::Context;


#[tracing::instrument(
    name = "Get a new cupom", skip(pool)
)]
#[get("cupom")]
pub async fn get_cupom(body: web::Json<CupomRequest>, pool: web::Data::<MySqlPool>) -> Result<HttpResponse, CupomError> {

    dbg!(body);

    let row: Option<_> = query!(
        r#"
        SELECT id, code, discount FROM cupom
        "#
    )
    .fetch_optional(pool.as_ref())
    .await
    .context("Failed to perform query")?
    .map(|row| (row.id, row.code, row.discount));

    // dbg!(row);
    match row {
        Some(row) => {
            let test = CupomResponse { id: row.0, code: row.1, discount: row.2};

            return Ok(HttpResponse::Ok().json(test));
        },
        _ => return Ok(HttpResponse::NoContent().finish()),
    };


}

#[tracing::instrument(
    name = "Post cupom", skip(pool)
)]
#[post("cupom")]
pub async fn post_cupom(request: web::Json<CupomRequest>, pool: web::Data::<MySqlPool>) -> Result<HttpResponse, CupomError> {
    let cupom = insert_cupom(request, &pool).await?;
    return Ok(HttpResponse::Created().json(cupom));
}




