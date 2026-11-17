use testcontainers::{ContainerAsync, runners::AsyncRunner};
use testcontainers_modules::postgres::Postgres;

pub async fn init_postgres() -> Result<(ContainerAsync<Postgres>, String), std::io::Error> {
    let postgres = Postgres::default()
        .start()
        .await
        .map_err(|e| std::io::Error::other(e))?;

    let port = postgres
        .get_host_port_ipv4(5432)
        .await
        .map_err(|e| std::io::Error::other(e))?;

    let url = format!("postgres://postgres:postgres@127.0.0.1:{port}/postgres");

    Ok((postgres, url))
}
