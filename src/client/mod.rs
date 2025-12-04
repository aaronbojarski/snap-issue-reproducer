use anyhow::Context;
use scion_stack::scionstack::ScionStackBuilder;
use tracing::{debug, info, warn};
use url::Url;

pub struct Client {
    remote: scion_proto::address::SocketAddr,
    endhost_api_address: Url,
    snap_token: Option<String>,
}

impl Client {
    pub fn new(
        remote: scion_proto::address::SocketAddr,
        endhost_api_address: Url,
        snap_token: Option<String>,
    ) -> Self {
        Self {
            remote,
            endhost_api_address,
            snap_token,
        }
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        let mut builder = ScionStackBuilder::new(self.endhost_api_address.clone());
        if let Some(snap_token) = &self.snap_token {
            builder = builder.with_auth_token(snap_token.clone());
        }
        let client_network_stack = builder.build().await?;

        let assigned_addr = client_network_stack
            .local_addresses()
            .first()
            .cloned()
            .context("client did not get any address assigned")?;

        let socket_address = scion_proto::address::SocketAddr::new(assigned_addr.into(), 10111);
        let socket = client_network_stack.bind(Some(socket_address)).await?;

        let mut send_interval = tokio::time::interval(std::time::Duration::from_millis(100));
        let mut send_count: u64 = 0;

        let mut buf = vec![0u8; 65536];
        loop {
            tokio::select! {
                // Receive datagram from socket
                Ok((len, src)) = socket.recv_from(&mut buf) => {
                    debug!("received {} bytes on socket from {}", len, src);
                    if len < 16 {
                        warn!("invalid packet received from {}: not enough data", src);
                        continue;
                    }
                    // Decode message with same format as sent
                    if buf[0..16] == b"Ping send_count="[..] {
                        match String::from_utf8_lossy(&buf[16..len])
                            .parse::<u64>()
                        {
                            Ok(count) => {
                                info!("received ping with send_count={}", count);
                            }
                            Err(e) => {
                                warn!("failed to parse send_count from {}, {}", src, e);
                            }
                        }
                    } else {
                        let received_message = String::from_utf8_lossy(&buf[..len]);
                        warn!("received unknown message format from {}, {}", src, received_message);
                    }

                }

                // Send ping on every tick
                _ = send_interval.tick() => {
                    let message = b"Ping send_count=".iter()
                        .chain(send_count.to_string().as_bytes())
                        .cloned()
                        .collect::<Vec<u8>>();
                    if socket.send_to(&message, self.remote).await.is_ok() {
                        debug!("sent {} bytes to {}", message.len(), self.remote);
                        info!("sent ping with send_count={}", send_count);
                        send_count += 1;
                    } else {
                        warn!("failed to send data to {}", self.remote);
                    }
                }
            }
        }
    }
}
