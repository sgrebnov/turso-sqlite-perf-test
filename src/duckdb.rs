use std::sync::Arc;
use anyhow::Result;
use duckdb::{arrow::array::RecordBatch, DuckdbConnectionManager};


pub struct DuckDbConnectionPool {
    pool: Arc<r2d2::Pool<DuckdbConnectionManager>>
}

impl DuckDbConnectionPool {
    pub fn new(database_path: &str) -> Result<Self, r2d2::Error> {
        let manager = DuckdbConnectionManager::file(database_path).unwrap();
        let pool_builder = r2d2::Pool::builder()
            .max_size(10);

        Ok(DuckDbConnectionPool {
            pool: Arc::new(pool_builder.build(manager).unwrap()),
        })
    }

    pub fn get_connection(&self) -> Result<r2d2::PooledConnection<DuckdbConnectionManager>, r2d2::Error> {
        self.pool.get()
    }
}

pub fn execute_query(pool: &DuckDbConnectionPool, sql: &str) -> Result<Vec<RecordBatch>> {
    let conn = pool.get_connection()
        .map_err(|e| anyhow::anyhow!("Failed to get connection from pool: {}", e))?;
    
    let mut stmt = conn.prepare(sql)
        .map_err(|e| anyhow::anyhow!("Failed to prepare statement: {}", e))?;

    let res: Vec<::duckdb::arrow::array::RecordBatch> = stmt.query_arrow([]).map(|iter| iter.collect())
        .map_err(|e| anyhow::anyhow!("Failed to collect results: {}", e))?;

    Ok(res)
}