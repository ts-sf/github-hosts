pub mod consts;
mod dns;

pub mod dns_server;
pub mod executor;

pub mod version;
pub mod util;

pub fn config_logger(log: &str) {
    use tracing_subscriber::prelude::*;
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(log))
        .with(tracing_subscriber::fmt::layer())
        .init();
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use hex_literal::hex;
    use tokio::net::UdpSocket;
    use trust_dns_client::{
        client::{AsyncClient, ClientHandle},
        op::{DnsResponse, Message},
        proto::{rr::{DNSClass, Name, RecordType}, xfer::DnsRequest},
        udp::UdpClientStream,
    };

    // #[tokio::test]
    #[tokio::test]
    async fn d() {
        let address = "8.8.8.8:53".parse().unwrap();
        // let conn = UdpClientConnection::new(address).unwrap();
        // let client = SyncClient::new(conn);

        let udp_stream = UdpClientStream::<UdpSocket>::new(address);
        let aclient = AsyncClient::connect(udp_stream);
        let (mut aclient, bg) = aclient.await.unwrap();
        tokio::spawn(bg);

        // Specify the name, note the final '.' which specifies it's an FQDN
        let name = Name::from_str("github.com.").unwrap();

        // NOTE: see 'Setup a connection' example above
        // Send the query and get a message response, see RecordType for all supported options
        let response = aclient
            .query(name, DNSClass::IN, RecordType::A)
            .await
            .unwrap();
        // let answers: &[Record] = response.answers();
        println!("{:?}", response);

        let msg = Message::from_vec(&hex!("589d818000010006000000000675706461746504636f64650c76697375616c73747564696f03636f6d0000010001c00c000500010000001e00301e7673636f64652d7570646174652d67366763623667676474686b63746439037a303107617a7572656664036e657400c03a000500010000001e002311737461722d617a75726566642d70726f640e747261666669636d616e61676572c065c076000500010000001e002a0473686564086475616c2d6c6f7709706172742d3030343506742d3030303908742d6d7365646765c065c0a5000500010000001e0002c0b3c0b3000100010000001e00040d6bd549c0b3000100010000001e00040d6bf649").to_vec()).unwrap();
        let dns = DnsResponse::from_message(msg).unwrap();
        
        let msg = Message::from_vec(&hex!("e2ed010000010000000000000675706461746504636f64650c76697375616c73747564696f03636f6d0000410001")).unwrap();
        let dns_req = DnsRequest::from(msg);

        println!("{:?}",dns_req.queries());
        
        println!(
            "\n\n{:?}",
            dns.answers().iter().filter_map(|x| match x.record_type() {
                RecordType::A => Some(x.clone().into_data().unwrap().into_a().unwrap()),
                _ => None,
            }).collect::<Vec<_>>()
        );
        ();
    }
}
