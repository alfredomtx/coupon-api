use actix_web::{
    web, get, post, delete, patch, put, HttpResponse, Responder,
    http:: {
        header::ContentType,
    },
};
use super::model::{Cupom, CupomRequest, CupomResponse, CupomError};
use super::cupom_service::{insert_cupom};
use sqlx::{
    query,
    MySqlPool,
};
use serde::{Serialize, Deserialize};
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
pub async fn post_cupom(request: web::Json<CupomRequest>, pool: web::Data::<MySqlPool>) -> impl Responder {
    match insert_cupom(request, &pool).await {
        Ok(cupom) => return HttpResponse::Created().json(cupom),
        Err(error) => {
            dbg!(error);
            return HttpResponse::BadRequest().finish();

        }
    }


}




