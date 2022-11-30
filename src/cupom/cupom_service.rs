use sqlx::{MySqlPool};
use actix_web::web::Json;
use anyhow::{Context, Result, anyhow};

use super::model::{CupomRequest, CupomResponse, CupomError, CupomInsert};
use super::cupom_repository;
use super::cupom_repository::Fields;


pub async fn get_all(pool: &MySqlPool) -> Result<Vec<CupomResponse>, CupomError> {
    let cupoms = cupom_repository::get_all(pool).await
        .map_err(|error| CupomError::UnexpectedError(error.into()))?;

    let cumpoms_response = cupoms
        .into_iter()
        // flat_map uses an iterator over the result of the mapping and as a consequence,
        // it will skip over elements for which the mapping closure returns empty or unsuccessful values
        .flat_map(|cupom| {
            match cupom.try_into() {
                Ok(cupom) => Some(cupom),
                Err(cupom) => {
                    tracing::error!("Failed to try_into() {:?}", cupom);
                    None
                }
            }
        })
        .collect();
    return Ok(cumpoms_response);
}

pub async fn get_by_id(id: i32, pool: &MySqlPool) -> Result<CupomResponse, CupomError> {
    let result = cupom_repository::get_by_id(id, pool).await
        .context("failed to get by id")?;

    let cupom = result.ok_or( CupomError::NotFoundError(anyhow!(format!("Cupom with id `{}` not found", id))))?;

    let cupom_response = cupom.try_into().map_err(CupomError::ValidationError)?;
    return Ok(cupom_response);
}

pub async fn get_by_code(code: String, pool: &MySqlPool) -> Result<CupomResponse, anyhow::Error> {
    let result = cupom_repository::get_by_field(Fields::Code(code.clone()), pool).await
        .map_err(|error| CupomError::UnexpectedError(error.into()))?;

    let cupom = result.ok_or(CupomError::NotFoundError(anyhow!(format!("Cupom with code `{}` not found", code))))?;

    let cupom_response = cupom.try_into().map_err(CupomError::ValidationError)?;
    return Ok(cupom_response);
}

pub async fn insert(cupom: Json<CupomRequest>, pool: &MySqlPool) -> Result<CupomResponse, anyhow::Error> {
    let cupom_insert = CupomInsert {
        code: cupom.code.to_string(),
        discount: cupom.discount,
    };

    let inserted_id = cupom_repository::insert(cupom_insert, pool).await
        .map_err(|error| CupomError::UnexpectedError(error.into()))?;

    let result = cupom_repository::get_by_id(inserted_id as i32, pool).await
        .map_err(|error| CupomError::UnexpectedError(error.into()))?;

    let cupom = result.ok_or(CupomError::NotFoundError(anyhow!("not found")))?;

    let cupom_response = cupom.try_into().map_err(CupomError::ValidationError)?;
    return Ok(cupom_response);
}
