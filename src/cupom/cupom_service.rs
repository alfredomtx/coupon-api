use anyhow::Ok;
use sqlx::{query, MySqlPool};
use actix_web::web::Json;

use super::model::{Cupom, CupomRequest, CupomResponse, CupomError};
use super::cupom_repository::*;

pub async fn insert_cupom(cupom: Json<CupomRequest>, pool: &MySqlPool) -> Result<CupomResponse, anyhow::Error> {
    let cupom_request = Cupom {
        code: String::from(&cupom.code),
        discount: cupom.discount,
    };

    let result = query!(
        r#"
            INSERT INTO cupom (code, discount)
            VALUES (?, ?)
        "#,
        String::from(&cupom_request.code), cupom_request.discount
    )
    .execute(pool)
    .await
    .map_err( |error| CupomError::UnexpectedError(error.into()))?;

    let inserted_id = result.last_insert_id() as i32;

    let result = get_by_id(inserted_id, pool)
        .await
        .map_err(|error| CupomError::UnexpectedError(error.into()))?;

    let cupom = result.ok_or_else(|| {
        return CupomError::NotFoundError(anyhow::anyhow!("Cupom not found."));
    })?;

    let cupom_response = CupomResponse { id: inserted_id, code: cupom.code, discount: cupom.discount};
    return Ok(cupom_response);
    // match result {
    //     Some(cupom) => {
            
    //     },
    //     None => {
    //         return Err(CupomError::NotFoundError(anyhow::anyhow!("Cupom not found.")));
    //     }
    // };



}