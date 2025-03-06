use tiberius::{Client, Config};
use tokio::net::TcpStream;
use tokio_util::compat::{Compat, TokioAsyncWriteCompatExt};
use std::{error::Error, ops::{Deref, DerefMut}};

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

pub async fn connection_pool(database: &str) -> Result<Client<Compat<TcpStream>>, Box<dyn std::error::Error>> {
    // It uses an ADO.NET connection string to connect to SQL Server.
    // Replace with your actual connection string
    let config = Config::from_ado_string(
        &"Server=tcp:db12877.public.databaseasp.net;User=db12877;Password=Snakesystem@09;TrustServerCertificate=true;Database={};".replace("{}", database),
    )?;

    let tcp = <TcpStream as tiberius::SqlBrowser>::connect_named(&config).await?;
    // let _ = tcp.set_nodelay(true);

    let client = Client::connect(config, tcp.compat_write()).await?;
    println!("Connected to SQL Server");
    // let _ = client.close().await;

    Ok(client)
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