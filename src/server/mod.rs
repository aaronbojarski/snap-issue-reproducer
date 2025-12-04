use anyhow::Context;
use scion_stack::scionstack::ScionStackBuilder;
use tracing::instrument::Instrument;
use tracing::{debug, info, warn};
use url::Url;

pub struct Server {
    listen_addr: scion_proto::address::SocketAddr,
    endhost_api_address: Url,
    snap_token: Option<String>,
}

impl Server {
    pub fn new(
        listen_addr: scion_proto::address::SocketAddr,
        endhost_api_address: Url,
        snap_token: Option<String>,
    ) -> Self {
        Self {
            listen_addr,
            endhost_api_address,
            snap_token,
        }
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        let mut builder = ScionStackBuilder::new(self.endhost_api_address.clone());
        if let Some(snap_token) = &self.snap_token {
            builder = builder.with_auth_token(snap_token.clone());
        }
        let scion_network_stack = builder
            .build()
            .in_current_span()
            .await
            .context("error building server SCION stack")?;

        let server_addr = scion_network_stack
            .local_addresses()
            .first()
            .cloned()
            .context("server did not get any address assigned")?;

        let socket_address = scion_proto::address::SocketAddr::new(server_addr.into(), 4433);
        let socket = scion_network_stack.bind(Some(socket_address)).await?;
        let local_scion_addr = socket.local_addr();
        info!("listening on {}", local_scion_addr);

        if local_scion_addr != self.listen_addr {
            warn!(
                "listening address {} differs from requested address {}",
                local_scion_addr, self.listen_addr
            );
        }

        let mut buf = vec![0u8; 65536];
        loop {
            // Receive datagram from UDP socket
            if let Ok((len, src)) = socket.recv_from(&mut buf).await {
                debug!("received {} bytes on socket from {}", len, src);

                if len == 0 {
                    warn!("received empty datagram from {}", src);
                    continue;
                }
                // Echo the datagram back to the sender
                if socket.send_to(&buf[..len], src).await.is_ok() {
                    debug!("sent {} bytes back to {}", len, src);
                } else {
                    warn!("failed to send response to {}", src);
                }
            }
        }
    }
}
