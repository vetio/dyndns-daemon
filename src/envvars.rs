use errors::*;

#[cfg(feature = "use_dotenv")]
pub fn use_dotenv() -> Result<()>{
    use dotenv;

    if let Err(ref e) = dotenv::dotenv() {
        bail!("failed to load .env file: {:?}", e);
    }
    Ok(())
}

#[cfg(not(feature = "use_dotenv"))]
pub fn use_dotenv() -> Result<()>{
    Ok(())
}
