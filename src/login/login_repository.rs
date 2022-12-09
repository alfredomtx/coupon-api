use sqlx::{MySqlPool, query};


pub async fn get_by_email(email: String, pool: &MySqlPool) -> Result<Option<(i32, String)>, sqlx::Error> {
    let result = query!(
        r#"
        SELECT id, role FROM user WHERE email = ?
        "#, email
    )
    .fetch_optional(pool)
    .await
    .map_err(|error| {
        tracing::error!("Failed to execute select query: {:?}", error);
        error
    })?;

    return Ok(result.map(|r| (r.id, r.role) ));
}

