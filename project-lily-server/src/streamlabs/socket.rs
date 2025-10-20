use std::sync::Arc;

use anyhow::Result;
use futures_util::FutureExt;
use log::{error, info, warn};
use rust_socketio::{
    Payload, TransportType,
    asynchronous::{Client, ClientBuilder},
};
use tokio::sync::{
    Mutex,
    mpsc::{UnboundedReceiver, UnboundedSender},
};

#[derive(Clone)]
pub struct SocketioConnection {
    /// The current connection
    client: Client,
    pub connected: Arc<Mutex<bool>>,
}

impl SocketioConnection {
    /// Connect to the websocket and return the stream
    pub async fn get_connection(
        transmitter: UnboundedSender<Payload>,
        token: &str,
    ) -> Result<SocketioConnection> {
        info!("connecting to socketio");

        let connected = Arc::new(Mutex::new(true));

        let connected_ = connected.clone();
        let socket = ClientBuilder::new(format!("wss://sockets.streamlabs.com?token={}", token))
            .on("event", move |msg, _| {
                let tx = transmitter.clone();

                async move {
                    info!("received streamlabs event: {:?}", msg);
                    tx.send(msg).unwrap();
                }
                .boxed()
            })
            .on("error", move |err, conn| {
                {
                    let connected__ = connected_.clone();
                    async move {
                        let mut lock = connected__.lock().await;
                        error!("Error: {:#?}", err);
                        if let Err(e) = conn.disconnect().await {
                            error!("Failed to disconnect after error: {:#?}", e);
                        }
                        *lock = false;
                    }
                }
                .boxed()
            })
            .transport_type(TransportType::Websocket)
            .connect()
            .await?;

        Ok(SocketioConnection {
            client: socket,
            connected,
        })
    }

    /// Handle a single iteration of the websocket connection
    /// Returns Ok(true) if you should continue running, Ok(false) if you should stop
    pub async fn run(
        &mut self,
        receiver: &mut UnboundedReceiver<Payload>,
    ) -> Result<(bool, Option<Payload>)> {
        if !self.connected.lock().await.clone() {
            info!("No longer connected to streamlabs websocket");
            return Ok((false, None));
        }

        if let Some(msg) = receiver.recv().await {
            return Ok((true, Some(msg)));
        } else {
            warn!("connection closed by server");
            return Ok((true, None));
        }
    }

    pub async fn disconnect(&mut self) -> Result<()> {
        info!("disconnecting from streamlabs websocket");
        if let Ok(()) = self.client.disconnect().await {
            info!("disconnected from streamlabs  websocket");
            let mut lock = self.connected.lock().await;
            *lock = false;
        }
        Ok(())
    }
}

impl std::fmt::Debug for SocketioConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SocketioConnection").finish()
    }
}
