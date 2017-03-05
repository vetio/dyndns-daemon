
#[cfg(use_dotenv)]
pub fn use_dotenv() {
    if let Err(ref e) = dotenv::dotenv() {
        bail!("failed to load .env file: {:?}", e);
    }
}

#[cfg(not(use_dotenv))]
pub fn use_dotenv() {}
