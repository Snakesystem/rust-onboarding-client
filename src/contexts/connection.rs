use bb8::Pool;
use bb8_tiberius::ConnectionManager;
use tiberius::{Client, Config};
use tokio::net::TcpStream;
use tokio_util::compat::{Compat, TokioAsyncWriteCompatExt};
use std::{error::Error, ops::{Deref, DerefMut},env};

pub struct BeginTransaction<'a> {
    client: &'a mut Client<Compat<TcpStream>>,
    committed: bool,
}

impl<'a> BeginTransaction<'a> {
    pub async fn new(client: &'a mut Client<Compat<TcpStream>>) -> Result<Self, Box<dyn Error>> {
        begin_transaction(client).await?;
        Ok(Self { client, committed: false })
    }

    pub async fn commit(mut self) -> Result<(), Box<dyn Error>> {
        commit_transaction(self.client).await?;
        self.committed = true;
        Ok(())
    }
}

// Deref supaya bisa langsung akses `client` dari `transaction`
impl<'a> Deref for BeginTransaction<'a> {
    type Target = Client<Compat<TcpStream>>;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

impl<'a> DerefMut for BeginTransaction<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.client
    }
}

// Rollback otomatis kalau lupa commit
impl<'a> Drop for BeginTransaction<'a> {
    fn drop(&mut self) {
        if !self.committed {
            println!("⚠️ Transaction rolled back automatically!");

            tokio::task::block_in_place(|| {
                let rt = tokio::runtime::Handle::current();
                let _ = rt.block_on(rollback_transaction(self.client));
            });
        }
    }
}

pub async fn create_pool(database: &str) -> Result<Pool<ConnectionManager>, Box<dyn std::error::Error + Send + Sync + 'static>> {
    let database_user = env::var("DATABASE_USER");
    let database_password = env::var("DATABASE_PASSWORD");
    let connection_string = format!(
        "Server=tcp:db12877.public.databaseasp.net;User={};Password={};TrustServerCertificate=true;Database={}",
        database_user.unwrap(),
        database_password.unwrap(),
        database
    );

    let config = Config::from_ado_string(&connection_string)?;
    let manager = ConnectionManager::new(config);
    let pool = Pool::builder().max_size(10).build(manager).await?;

    Ok(pool)
}

pub async  fn begin_transaction(client: &mut Client<Compat<TcpStream>>) -> Result<(), Box<dyn std::error::Error>> {
    client.simple_query("BEGIN TRANSACTION").await?;
    Ok(())
}

pub async fn commit_transaction(client: &mut Client<Compat<TcpStream>>) -> Result<(), Box<dyn std::error::Error>> {
    client.simple_query("COMMIT").await?;
    Ok(())
}

pub async fn rollback_transaction(client: &mut Client<Compat<TcpStream>>) -> Result<(), Box<dyn std::error::Error>> {
    client.simple_query("ROLLBACK").await?;
    Ok(())
}