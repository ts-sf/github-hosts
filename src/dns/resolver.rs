use trust_dns_resolver::{
    config::{ResolverConfig, ResolverOpts},
    TokioAsyncResolver,
};

pub async fn resolve_domain(domain: &str) -> anyhow::Result<Vec<String>> {
    // Construct a new Resolver with default configuration options
    let resolver = TokioAsyncResolver::tokio(ResolverConfig::cloudflare(), ResolverOpts::default());

    // Lookup the IP addresses associated with a name.
    let response = resolver.lookup_ip(domain).await?;

    // There can be many addresses associated with the name,
    //  this can return IPv4 and/or IPv6 addresses
    let address = response
        .iter()
        .map(|ip| ip.to_string())
        .collect::<Vec<String>>();

    Ok(address)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_resolve_domain() {
        let domain = "example.com";
        let ip = resolve_domain(domain).await.unwrap();
        println!("domain: {}, ip: {:?}", domain, ip);
    }
}
