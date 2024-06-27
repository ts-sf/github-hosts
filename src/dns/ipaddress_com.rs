use std::{collections::HashSet, time::Duration};

use super::{IPV6_ADDR_REGEX, IP_ADDR_REGEX};
use regex::Regex;

const DNS_HOST: &str = "https://www.ipaddress.com/site";
const HTML_LOCAL_STR: &str = r#"@type":"Answer","text":"The hostname resolves to"#;

pub async fn ipaddress_com_records(domain: &str, timeout: Duration) -> anyhow::Result<Vec<String>> {
    let url = format!("{}/{}", DNS_HOST, domain);
    let client = reqwest::Client::new();

    let body = client
        .get(&url)
        .timeout(timeout)
        .send()
        .await?
        .text()
        .await?;
    let index = body.find(HTML_LOCAL_STR).ok_or(anyhow::anyhow!("body: {body}"))?;

    let reg = format!("<em>({})</em>", IP_ADDR_REGEX);
    let re = Regex::new(&reg).unwrap();
    let mut records = HashSet::new();

    for ip in re
        .captures_iter(&body[index..])
        .filter_map(|c| c.get(1).map(|m| m.as_str()))
    {
        records.insert(ip.to_string());
    }

    let reg = format!("<em>({})</em>", IPV6_ADDR_REGEX);
    let re = Regex::new(&reg).unwrap();
    // let mut v6_records = HashSet::new();
    for _ip in re
        .captures_iter(&body[index..])
        .filter_map(|c| c.get(1).map(|m| m.as_str()))
    {
        // TODO add ipv6 also.
        // records.insert(_ip.to_string());
    }

    Ok(records.into_iter().collect())
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::config_logger;
    #[tokio::test]
    async fn test_records() {
        config_logger("info");
        let result = ipaddress_com_records("raw.github.com", Duration::from_secs(10)).await;
        tracing::info!("{result:?}");
    }
}
