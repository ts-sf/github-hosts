use std::{net::SocketAddr, sync::Arc};

use tokio::{net::UdpSocket, sync::mpsc, task::JoinSet};

pub async fn server(server_at: &str, remote_dns_addr: &str) -> anyhow::Result<()> {
    let sock = UdpSocket::bind(server_at).await.unwrap();

    let sock = Arc::new(sock);

    let (tx, mut rx) = mpsc::unbounded_channel();

    let inner_sock = sock.clone();
    let _handle = tokio::spawn(async move {
        let mut buf = [0; 1024];
        loop {
            if let Ok((len, addr)) = inner_sock.recv_from(&mut buf).await {
                tracing::debug!(
                    "{:?} bytes received from {:?}: {:?}",
                    len,
                    addr,
                    &buf[0..len]
                );

                let _ = tx.send((addr, buf[0..len].to_vec()));
            }
        }
    });

    let mut tasks = JoinSet::new();

    loop {
        tokio::select! {
            Some((addr,buf)) = rx.recv() => {
                let inner_sock = sock.clone();
                let romote_addr = remote_dns_addr.to_string();
                tasks.spawn(forward(inner_sock, addr, buf,romote_addr));
            }

            Some(_t) = tasks.join_next() => {
            }
        }
    }

    #[allow(unreachable_code)]
    Ok(())
}

async fn forward(
    response_sock: Arc<UdpSocket>,
    to: SocketAddr,
    buf: Vec<u8>,
    remote_dns_addr: String,
) -> anyhow::Result<()> {
    let client = UdpSocket::bind("0.0.0.0:0").await.unwrap();
    let reomte_dns = remote_dns_addr.parse::<SocketAddr>().unwrap();
    client.connect(reomte_dns).await?;

    use trust_dns_client::{
        op::Message,
        proto::xfer::{DnsRequest, DnsResponse},
        rr::RecordType,
    };
    if let Ok(msg) = Message::from_vec(&buf) {
        let req = DnsRequest::from(msg);
        let queries = req.queries();

        tracing::info!("new query: {:?}", queries);
    }

    match client.send(&buf).await {
        Ok(_) => {
            let mut buf = [0; 1024];
            if let Ok(len) = client.recv(&mut buf).await {
                response_sock.send_to(&buf[0..len], to).await?;

                if let Ok(msg) = Message::from_vec(&buf) {
                    if let Ok(res) = DnsResponse::from_message(msg) {
                        tracing::info!(
                            "new response: {:?}",
                            res.answers()
                                .iter()
                                .filter_map(|x| match x.record_type() {
                                    RecordType::A =>
                                        Some(x.clone().into_data().unwrap().into_a().unwrap()),
                                    _ => None,
                                })
                                .collect::<Vec<_>>()
                        );
                    }
                }
            } else {
                tracing::warn!("recv error.");
            }
        }
        Err(e) => {
            tracing::warn!("send pack error: {}", e.to_string());
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::config_logger;

    use super::*;

    #[tokio::test]
    #[ignore = "server thread."]
    async fn test_server() {
        config_logger("info");
        server("127.0.0.1", "8.8.8.8:53").await.unwrap();
    }
}
