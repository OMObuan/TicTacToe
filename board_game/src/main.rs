use board_game::{
    game_executor::GameExecutor,
    game_manager,
    message::{ExecutorToManagerMsg, ManagerToExecutorMsg},
};
use tokio::{join, sync::mpsc};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
            .with_max_level(tracing::Level::TRACE)
            .finish(),
    )?;
    let (manager_to_executor_tx, manager_to_executor_rx) = mpsc::channel(32);
    let (executor_to_manager_tx, executor_to_manager_rx) = mpsc::channel(32);

    let mut game_manager = game_manager::GameManager::new()
        .set_rx(executor_to_manager_rx)
        .set_tx(manager_to_executor_tx);

    struct RealGameExecutor {
        tx: Option<mpsc::Sender<ExecutorToManagerMsg>>,
        rx: Option<mpsc::Receiver<ManagerToExecutorMsg>>,
    }

    impl RealGameExecutor {
        fn new() -> Self {
            Self { tx: None, rx: None }
        }

        fn set_rx(mut self, rx: mpsc::Receiver<ManagerToExecutorMsg>) -> Self {
            self.rx = Some(rx);
            self
        }

        fn set_tx(mut self, tx: mpsc::Sender<ExecutorToManagerMsg>) -> Self {
            self.tx = Some(tx);
            self
        }
    }

    impl GameExecutor for RealGameExecutor {
        fn draw_opening_screen(&mut self) {}

        fn get_channel(
            &mut self,
        ) -> (
            &mpsc::Sender<ExecutorToManagerMsg>,
            &mut mpsc::Receiver<ManagerToExecutorMsg>,
        ) {
            (self.tx.as_ref().unwrap(), self.rx.as_mut().unwrap())
        }
    }

    let mut game_executor = RealGameExecutor::new()
        .set_rx(manager_to_executor_rx)
        .set_tx(executor_to_manager_tx);

    let game_executor_task = tokio::spawn(async move { game_executor.run().await.unwrap() });

    let game_manager_task = tokio::spawn(async move {
        game_manager.start().await.unwrap();
    });

    join!(game_executor_task, game_manager_task).0?;
    Ok(())
}
