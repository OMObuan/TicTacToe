use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use board_game::{
    game_executor::{GameExecutor, GameExecutorError},
    game_manager::{self, GameManagerError},
    message::{
        ExecutorToManagerMsg, ManagerToExecutorMsg, request_message::ExecutorToManagerReqMsg,
        response_message::ExecutorToManagerResMsg,
    },
};
use tokio::{
    join,
    sync::{Mutex, mpsc},
    task::JoinHandle,
    time::sleep,
};
use tracing::trace;
use tracing_appender::non_blocking::NonBlocking;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let file_appender = tracing_appender::rolling::daily("./log", "game.log");
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

    let game_manager = game_manager::GameManager::new()
        .set_rx(executor_to_manager_rx)
        .set_tx(manager_to_executor_tx);

    struct RealGameExecutor {
        tx: Option<Arc<Mutex<mpsc::Sender<ExecutorToManagerMsg>>>>,
        rx: Option<Arc<Mutex<mpsc::Receiver<ManagerToExecutorMsg>>>>,
    }

    impl RealGameExecutor {
        fn new() -> Self {
            Self { tx: None, rx: None }
        }

        fn set_rx(mut self, rx: mpsc::Receiver<ManagerToExecutorMsg>) -> Self {
            self.rx = Some(Arc::new(Mutex::new(rx)));
            self
        }

        fn set_tx(mut self, tx: mpsc::Sender<ExecutorToManagerMsg>) -> Self {
            self.tx = Some(Arc::new(Mutex::new(tx)));
            self
        }
    }

    #[async_trait]
    impl GameExecutor for RealGameExecutor {
        async fn draw_opening_screen(&self) -> Result<(), GameExecutorError> {
            Ok(())
        }

        fn get_tx(
            &self,
        ) -> Result<Arc<Mutex<mpsc::Sender<ExecutorToManagerMsg>>>, GameExecutorError> {
            Ok(self.tx.clone().unwrap())
        }

        fn get_rx(
            &self,
        ) -> Result<Arc<Mutex<mpsc::Receiver<ManagerToExecutorMsg>>>, GameExecutorError> {
            Ok(self.rx.clone().unwrap())
        }

        async fn init_game(&self) -> Result<(), GameExecutorError> {
            let tx = self.get_tx()?;
            trace!("init game");
            tx.lock()
                .await
                .send(ExecutorToManagerMsg::Response(
                    ExecutorToManagerResMsg::InitGameResponse,
                ))
                .await
                .unwrap();

            Ok(())
        }

        async fn quit_game(&self) -> Result<(), GameExecutorError> {
            let tx = self.get_tx()?;
            trace!("execute quiting game");
            tx.lock()
                .await
                .send(ExecutorToManagerMsg::Response(
                    ExecutorToManagerResMsg::QuitGameResponse,
                ))
                .await?;
            Ok(())
        }

        async fn execute_game(&self) -> Result<(), GameExecutorError> {
            let tx = self.get_tx()?;
            sleep(Duration::from_secs(2)).await;
            tx.lock()
                .await
                .send(ExecutorToManagerMsg::Request(
                    ExecutorToManagerReqMsg::ReadyToQuitGameRequest,
                ))
                .await
                .unwrap();
            Ok(())
        }
    }

    let game_executor = RealGameExecutor::new()
        .set_rx(manager_to_executor_rx)
        .set_tx(executor_to_manager_tx);

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
