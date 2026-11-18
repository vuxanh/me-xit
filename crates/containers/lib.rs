use testcontainers::{ContainerAsync, GenericImage, ImageExt, core::WaitFor, runners::AsyncRunner};

pub type Container = ContainerAsync<GenericImage>;

pub async fn setup_redis_container()
-> Result<(ContainerAsync<GenericImage>, String), testcontainers::TestcontainersError> {
    let container = GenericImage::new("redis", "8.10-alpine")
        .with_wait_for(WaitFor::message_on_stdout(
            "Ready to accept connections tcp",
        ))
        .start()
        .await?;

    let port = container.get_host_port_ipv4(6379).await?;

    let host = container.get_host().await?;

    let url = format!("redis://{host}:{port}");

    Ok((container, url))
}

pub async fn setup_postgres_container()
-> Result<(ContainerAsync<GenericImage>, String), testcontainers::TestcontainersError> {
    let container = GenericImage::new("postgres", "18.7-alpine")
        .with_wait_for(WaitFor::message_on_stdout(
            "database system is ready to accept connections",
        ))
        .with_env_var("POSTGRES_USER", "postgres")
        .with_env_var("POSTGRES_PASSWORD", "postgres")
        .start()
        .await?;

    let port = container.get_host_port_ipv4(5432).await?;

    let host = container.get_host().await?;

    let url = format!("postgres://postgres:postgres@{host}:{port}/postgres?sslmode=disable");

    Ok((container, url))
}
