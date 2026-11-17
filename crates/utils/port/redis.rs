use testcontainers::{ContainerAsync, runners::AsyncRunner};
use testcontainers_modules::redis::Redis;

pub async fn init_redis() -> Result<(ContainerAsync<Redis>, String), std::io::Error> {
    let redis = Redis::default()
        .start()
        .await
        .map_err(|e| std::io::Error::other(e))?;

    let port = redis
        .get_host_port_ipv4(5432)
        .await
        .map_err(|e| std::io::Error::other(e))?;

    let url = format!("postgres://postgres:postgres@127.0.0.1:{port}/postgres");

    Ok((redis, url))
}
