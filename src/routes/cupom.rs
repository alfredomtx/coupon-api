use actix_web::{
    web, get, HttpResponse, ResponseError,
    http::StatusCode,
};
use sqlx::{
    query,
    MySqlPool,
};
use serde::{Serialize, Deserialize};
use anyhow::Context;

#[derive(Deserialize, Debug)]
pub struct CupomRequest {
   code: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CupomResponse {
    id: i32,
    code: String,
}

#[tracing::instrument(
    name = "Get a new cupom",
    skip(pool)
)]
#[get("cupom")]
pub async fn get_cupom(body: web::Json<CupomRequest>, pool: web::Data::<MySqlPool>) -> Result<HttpResponse, CupomError>{

    dbg!(body);

    let row: Option<_> = query!(
        r#"
        SELECT * FROM cupom
        "#
    )
    .fetch_optional(pool.as_ref())
    .await
    .context("Failed to perform query")?
    .map(|row| (row.id, row.code));

    dbg!(row);

    let test = CupomResponse { id: 123, code: "test".to_string()};

    return Ok(HttpResponse::Ok().json(test));
}




// Same logic to get the full error chain on `Debug` 
impl std::fmt::Debug for CupomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return error_chain_fmt(self, f);
    }
}


#[derive(thiserror::Error)]
pub enum CupomError {
    #[error("Something went wrong.")]
    GenericError(#[source] anyhow::Error),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl ResponseError for CupomError {
    fn status_code(&self) -> StatusCode {
        match self {
            CupomError::GenericError(_) => StatusCode::BAD_REQUEST,
            CupomError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}


pub fn error_chain_fmt(
    e: &impl std::error::Error,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    writeln!(f, "{}\n", e)?;
    let mut current = e.source();
    while let Some(cause) = current {
        writeln!(f, "Caused by:\n\t{}", cause)?;
        current = cause.source();
    }
    Ok(())
}
