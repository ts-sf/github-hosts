use std::{
    borrow::Borrow,
    collections::{BTreeMap, HashMap},
    time::{Duration, Instant},
};

use futures::{stream::FuturesUnordered, StreamExt};
use tokio::net::TcpSocket;

// use crate::dns::ipaddress_com::ipaddress_com_records;
use crate::dns::resolver;

#[derive(Debug, Default, Clone)]
pub struct Executor {
    _domains: HashMap<String, Vec<String>>,
    _time_delays: HashMap<String, u32>,
}

impl Executor {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn resolve<'a, K: Borrow<str> + 'a, I: Iterator<Item = &'a K>>(&mut self, urls: I) {
        let mut futs = urls
            .map(|dm| async move {
                let mut retries = 0;
                let dm = dm.borrow();
                loop {
                    let r = resolver::resolve_domain(dm).await;
                    match r {
                        Ok(ips) => {
                            tracing::debug!("Resolved {dm}.");
                            break Ok((dm.to_owned(), ips));
                        }
                        Err(e) => {
                            retries += 1;
                            tracing::warn!("[{dm}] error {}", e.to_string());
                            if retries > 3 {
                                tracing::error!("Many attempts to resolve {dm} it failed: {e:?}");
                                return Err(());
                            }
                            tokio::time::sleep(Duration::from_millis(500)).await;
                        }
                    }
                }
            })
            .collect::<FuturesUnordered<_>>();

        while let Some(Ok((domain, ips))) = futs.next().await {
            tracing::info!("{domain}: {ips:?}");
            for ip in ips.iter() {
                self._time_delays.insert(ip.to_string(), u32::MAX);
            }

            self._domains.insert(domain, ips);
        }
    }

    pub async fn fresh_delays(&mut self) {
        let mut delays = self
            ._time_delays
            .keys()
            .map(|ip| {
                let inner_ip = ip.to_string();
                async {
                    let soc = TcpSocket::new_v4().unwrap();
                    let addr = format!("{}:443", inner_ip).parse().unwrap();
                    let timeout = Duration::from_secs(5);
                    let now = Instant::now();
                    match tokio::time::timeout(timeout, soc.connect(addr)).await {
                        Ok(Ok(_)) => {
                            let dur = now.elapsed();
                            (dur.as_micros() as u32, inner_ip)
                        }
                        Ok(_) => (u32::MAX - 2, inner_ip),
                        Err(_) => (u32::MAX - 1, inner_ip),
                    }
                }
            })
            .collect::<FuturesUnordered<_>>();

        while let Some((sec, ip)) = delays.next().await {
            tracing::info!("{sec:?}: {ip:?}");
            self._time_delays.entry(ip).and_modify(|r| *r = sec);
        }
    }

    pub fn best_answer(&self) -> BTreeMap<String, (String, u32)> {
        self._domains
            .iter()
            .filter_map(|(domain, list)| {
                let mut ips = list
                    .iter()
                    .filter_map(|ip| self._time_delays.get(ip).map(|delay| (ip.clone(), *delay)))
                    .collect::<Vec<_>>();
                ips.sort_by_key(|(_, delay)| *delay);
                ips.pop().map(|x| (domain.clone(), x))
            })
            .collect::<BTreeMap<_, _>>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{config_logger, consts::GITHUB_URLS};

    #[tokio::test]
    async fn executor() {
        config_logger("info");
        let mut e = Executor::default();
        e.resolve(GITHUB_URLS.iter()).await;

        e.fresh_delays().await;
        let hosts = e.best_answer();
        assert_eq!(hosts.len(), GITHUB_URLS.len());
    }
}
