use tokio::sync::mpsc;

use crate::telemetry::Telemetry;

mod games;

use games::GameBackend;

pub async fn main(tx: mpsc::Sender<Telemetry>) {
    let mut backend = Backend::new(tx);

    loop {
        backend.process().await;
    }
}

struct Backend {
    tx: mpsc::Sender<Telemetry>,
    game_backend: Option<Box<dyn GameBackend + Send>>,
}

impl Backend {
    fn new(tx: mpsc::Sender<Telemetry>) -> Self {
        Self {
            tx,
            game_backend: None,
        }
    }

    async fn process(&mut self) {
        if let Some(game_backend) = self.game_backend.as_mut() {
            tokio::select! {
                _ = tokio::time::sleep(tokio::time::Duration::from_millis(500)) => {
                    debug!("Game backend has timed out!");
                    self.game_backend = None;
                },
                result = game_backend.next_event() => {
                    if let Some(telemetry) = result {
                        if self.tx.capacity() == self.tx.max_capacity() {
                            if let Err(e) = self.tx.send(telemetry).await {
                                error!("Error sending telemetry: {:?}", e);
                                tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;
                            }
                        }
                    } else {
                        debug!("Game backend has returned None! If the game backend stops reporting data to the backend, the game backend is considered no longer working and we will look for a new backend.");
                        self.game_backend = None;
                    }
                }
            }
        } else {
            tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;
            self.game_backend = games::find_next_backend().await;
            if self.tx.capacity() == self.tx.max_capacity() {
                if let Err(e) = self.tx.send(Telemetry::default()).await {
                    error!("Error sending telemetry: {:?}", e);
                }
            }
        }
    }
}
