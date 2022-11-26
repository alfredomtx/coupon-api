use sqlx::{MySqlPool, query};

use super::model::Cupom;


pub async fn get_by_id(id: i32, pool: &MySqlPool) -> Result<Option<Cupom>, sqlx::Error> {
    let result = query!(
        r#"
            SELECT * FROM cupom WHERE id = ?
        "#,
        id
    )
    .fetch_optional(pool)
    .await
    .map_err(|error| {
        tracing::error!("Failed to execute query: {:?}", error);
        error
    })?;

    match result {
        Some(cupom) => {
            let cupom = Cupom { code: cupom.code, discount: cupom.discount };
            return Ok(Some(cupom));
        },
        None => { 
            return Ok(None);
        }
    }

}