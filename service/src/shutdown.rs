use std::sync::Arc;

pub async fn signal(manager: Arc<super::manager::Manager>) {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {
            log::info!("Shutting down (user-requested)");
            manager.close().await.expect("failed to install signal handler");
        },
        () = terminate => {
            log::info!("Shutting down (terminated)");
        },
    }
}
