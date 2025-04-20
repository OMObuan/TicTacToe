use std::sync::Arc;

use board_game::{
    game_executor::{GameExecutor, GameExecutorError},
    game_manager::{self, GameManagerError},
};
use tokio::{join, sync::mpsc, task::JoinHandle};
use tracing_appender::non_blocking::NonBlocking;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt as _, util::SubscriberInitExt as _};
use tui_game::TuiGameExecutor;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let file_appender = tracing_appender::rolling::daily("./logs", "game.log");
    let (non_blocking, _guard) = NonBlocking::new(file_appender);

    tracing_subscriber::registry()
        .with(
            fmt::layer()
                .with_writer(non_blocking)
                .with_file(true)
                .with_line_number(true)
                .with_level(true),
        )
        .with(EnvFilter::from_default_env().add_directive("trace".parse().unwrap()))
        .init();
    let (manager_to_executor_tx, manager_to_executor_rx) = mpsc::channel(32);
    let (executor_to_manager_tx, executor_to_manager_rx) = mpsc::channel(32);

    let game_manager = Arc::new(
        game_manager::GameManager::new()
            .set_rx(executor_to_manager_rx)
            .set_tx(manager_to_executor_tx),
    );

    let game_executor = Arc::new(
        TuiGameExecutor::new()
            .set_rx(manager_to_executor_rx)
            .set_tx(executor_to_manager_tx),
    );

    let game_executor_task: JoinHandle<Result<(), GameExecutorError>> = tokio::spawn(async move {
        game_executor.run().await?;
        Ok(())
    });

    let game_manager_task: JoinHandle<Result<(), GameManagerError>> = tokio::spawn(async move {
        game_manager.start().await?;
        Ok(())
    });

    join!(game_executor_task, game_manager_task).0??;
    Ok(())
}
