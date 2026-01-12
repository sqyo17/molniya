use std::env;
use anyhow::Result;
use mysql::Pool;
use mysql::prelude::Queryable;

pub fn mysql_pool() -> Result<Pool> {
    let user = env::var("MYSQL_USER")
        .map_err(|_| anyhow::anyhow!("MYSQL_USER is not set"))?;

    let password = env::var("MYSQL_PASSWORD").unwrap_or_default();
    let host = env::var("MYSQL_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("MYSQL_PORT").unwrap_or_else(|_| "3306".to_string());

    let auth = if password.is_empty() {
        user
    } else {
        format!("{}:{}", user, password)
    };

    let url = format!("mysql://{}@{}:{}", auth, host, port);
    Ok(Pool::new(url.as_str())?)

}

pub fn test_connection() -> Result<()> {
    let pool = mysql_pool()?;
    let mut conn = pool.get_conn()?;

    // Simple no-op query
    conn.query_drop("SELECT 1")?;

    Ok(())
}