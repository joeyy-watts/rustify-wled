use dns_lookup::lookup_host;

pub fn resolve_ip(host: &str) -> Result<String, &'static str> {
    match lookup_host(host) {
        // there should only be one
        Ok(ips) => {
            return Ok(ips[0].to_string());
        },
        Err(_) => {
            return Err("Failed to resolve IP");
        }
    }
}