use std::{net::SocketAddr, str::FromStr};

use trust_dns_client::{
    op::{Message, Query},
    proto::xfer::{DnsRequest, DnsResponse},
    rr::{Name, RecordType},
};

use tokio::net::UdpSocket;

pub async fn query_dns(
    dns_server: &str,
    domain: &str,
    record_type: RecordType,
) -> anyhow::Result<DnsResponse> {
    let mut request = Message::new();

    let name = Name::from_str(domain)?;
    let query = Query::query(name, record_type);

    request.add_query(query);

    let request = DnsRequest::new(request, Default::default());
    let buf = request.to_vec()?;

    let client = UdpSocket::bind("0.0.0.0:0").await.unwrap();
    let reomte_dns = dns_server.parse::<SocketAddr>().unwrap();
    client.connect(reomte_dns).await?;

    match client.send(&buf).await {
        Ok(_) => {
            let mut buf = [0; 1024];
            if let Ok(_len) = client.recv(&mut buf).await {
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

                        tracing::info!("response: {:?}", res);

                        return Ok(res);
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

    anyhow::bail!("query dns failed.");
}

#[cfg(test)]
mod tests {
    use trust_dns_client::op::ResponseCode;

    use crate::config_logger;

    use super::*;

    #[tokio::test]
    async fn test_query_dns() {
        config_logger("info");
        let response = query_dns("1.1.1.1:53", "github.com.", RecordType::A)
            .await
            .unwrap();
        response
            .answers()
            .iter()
            .for_each(|x| tracing::info!("answer: {:?}", x));
        assert_eq!(response.response_code(), ResponseCode::NoError);
    }
}
