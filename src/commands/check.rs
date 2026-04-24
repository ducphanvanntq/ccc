use anyhow::{bail, Result};

use crate::api::{check_api_key, get_api_config, get_current_key};
use crate::utils::mask_key;

pub fn run() -> Result<()> {
    let api_key = match get_current_key() {
        Some(key) => key,
        None => bail!("API key not set. Run 'ccc key add' first."),
    };

    let (base_url, model) = get_api_config();

    println!("Checking API connection...");
    println!("  URL:   {base_url}/v1/messages");
    println!("  Model: {model}");
    println!("  Key:   {}", mask_key(&api_key));
    println!();

    let (ok, msg) = check_api_key(&api_key);
    if ok {
        println!("  [OK] API key is valid!");
    } else {
        println!("  [!!] API check failed: {msg}");
    }

    println!();
    Ok(())
}
