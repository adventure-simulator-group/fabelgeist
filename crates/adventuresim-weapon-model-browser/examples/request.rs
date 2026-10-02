//! Read one JSON request from stdin and write the canonical JSON result.
use std::io::{Read, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut request = String::new();
    std::io::stdin().read_to_string(&mut request)?;
    let response = adventuresim_weapon_model_browser::weapon_model_request(&request)?;
    std::io::stdout().write_all(response.as_bytes())?;
    Ok(())
}
