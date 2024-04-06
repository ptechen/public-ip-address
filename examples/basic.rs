use public_ip_address::lookup::LookupProvider;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let result =
        public_ip_address::perform_cached_lookup_with(vec![LookupProvider::MyIp], Some(2), false)?;
    println!(
        "Hello {} from {}, {}.",
        result.ip,
        result.city.unwrap_or("unknown".to_string()),
        result.country.unwrap_or("unknown".to_string())
    );
    Ok(())
}
