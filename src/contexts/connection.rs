use bb8::{Pool, PooledConnection};
use bb8_tiberius::ConnectionManager;
use tiberius::Config;
use tokio::sync::Mutex;
use std::{env, sync::Arc};

pub type DbPool = Pool<ConnectionManager>;
pub struct DbTransaction<'a> {
    pub conn: Arc<Mutex<Option<PooledConnection<'a, ConnectionManager>>>>, // 🔥 Pakai lifetime 'a
    committed: bool,
}

impl<'a> DbTransaction<'a> {
    pub async fn begin(pool: &'a Pool<ConnectionManager>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = pool.get().await?; // ✅ Tidak pakai 'static, langsung gunakan 'a
        conn.simple_query("BEGIN TRANSACTION").await?; // Mulai transaksi

        Ok(Self {
            conn: Arc::new(Mutex::new(Some(conn))),
            committed: false,
        })
    }

    pub async fn commit(mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut conn_guard = self.conn.lock().await;
        if let Some(mut conn) = conn_guard.take() {
            conn.simple_query("COMMIT").await?;
        }
        self.committed = true;
        Ok(())
    }
}

impl<'a> Drop for DbTransaction<'a> {
    fn drop(&mut self) {
        if !self.committed {
            // Ambil kunci secara langsung tanpa `tokio::spawn()`
            let mut conn_guard = self.conn.blocking_lock();
            if let Some(mut conn) = conn_guard.take() {
                // Rollback transaksi langsung dalam thread ini
                let runtime = tokio::runtime::Runtime::new().unwrap();
                let _ = runtime.block_on(async {
                    let _ = conn.simple_query("ROLLBACK").await;
                });
            }
        }
    }
}

/// Membuat pool koneksi database
pub async fn create_pool(database: &str) -> Result<DbPool, Box<dyn std::error::Error + Send + Sync>> {
    let database_user = env::var("DATABASE_USER").expect("DATABASE_USER harus diatur");
    let database_password = env::var("DATABASE_PASSWORD").expect("DATABASE_PASSWORD harus diatur");

    let connection_string = format!(
        "Server=tcp:db12877.public.databaseasp.net;User={};Password={};TrustServerCertificate=true;Database={}",
        database_user, database_password, database
    );

    let config = Config::from_ado_string(&connection_string)?;
    let manager = ConnectionManager::new(config);
    let pool = Pool::builder().max_size(10).build(manager).await?;

    Ok(pool)
}