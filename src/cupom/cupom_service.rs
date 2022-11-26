use sqlx::{query, MySqlPool};
use actix_web::web::Json;

use super::model::{Cupom, CupomRequest, CupomResponse, CupomError};

pub async fn insert_cupom(cupom: Json<CupomRequest>, pool: &MySqlPool) -> Result<CupomResponse, sqlx::Error> {
    let cupom_request = Cupom {
        code: String::from(&cupom.code),
        discount: cupom.discount,
    };

    let new_cupom = sqlx::query!(
        r#"
            INSERT INTO cupom (code, discount)
            VALUES (?, ?)
        "#,
        String::from(&cupom_request.code), cupom_request.discount
    )
    .execute(pool)
    .await?;

    dbg!(new_cupom);

    let cupom_response = CupomResponse {
        id: 1,
        code: String::from(&cupom.code),
        discount: cupom.discount,
    };
    
    return Ok(cupom_response);

}