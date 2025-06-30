use limbo::Builder;
use std::future::Future;

pub struct LimboConnectionManager {
    database_path: String,
}

impl LimboConnectionManager {
    pub fn new(database_path: &str) -> Self {
        LimboConnectionManager {
            database_path: database_path.to_string(),
        }
    }
}

impl bb8::ManageConnection for LimboConnectionManager {
    type Connection = limbo::Connection;
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn connect(&self) -> impl Future<Output = Result<Self::Connection, Self::Error>> + Send {
        let database_path = self.database_path.clone();
        Box::pin(async move {
            let db = Builder::new_local(&database_path).build().await?;
            let conn = db.connect()?;
            Ok(conn)
        })
    }

    fn is_valid(
        &self,
        conn: &mut Self::Connection,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send {
        Box::pin(async move {
            conn.query("SELECT 1", ()).await?;
            Ok(())
        })
    }

    fn has_broken(&self, _conn: &mut Self::Connection) -> bool {
        false
    }
}

pub struct SqliteConnectionPool {
    pool: bb8::Pool<LimboConnectionManager>,
}

impl SqliteConnectionPool {
    pub async fn new(database_path: &str) ->  anyhow::Result<Self> {
        let manager = LimboConnectionManager::new(database_path);
        let pool = bb8::Pool::builder()
            .max_size(10)
            .build(manager)
            .await.map_err(|e| {
                anyhow::anyhow!("Failed to create connection pool: {}", e)
            })?;
        
        Ok(SqliteConnectionPool { pool })
    }

    pub async fn get_connection(&self) ->  anyhow::Result<bb8::PooledConnection<'_, LimboConnectionManager>> {
        self.pool.get().await.map_err(|e| {
            anyhow::anyhow!("Failed to get connection from pool: {:?}", e)
        })
    }
}

pub async fn execute_query(pool: &SqliteConnectionPool, sql: &str) -> anyhow::Result<Vec<limbo::Row>> {
    let conn = pool.get_connection().await?;
    let mut stmt = conn.prepare(sql).await?;
    let mut rows = stmt.query(()).await?;
    let mut records = Vec::new();

    while let Some(batch) = rows.next().await? {
        records.push(batch);
    }
    Ok(records)
}