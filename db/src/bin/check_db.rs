use diesel::prelude::*;
use eyre::{Result, eyre}; // for Connection::establish

fn main() -> Result<()> {
    // Load .env if present
    let _ = dotenvy::dotenv();

    // Print URL for debugging
    let url = std::env::var("DATABASE_URL").map_err(|e| eyre!("DATABASE_URL not set: {e}"))?;
    println!("Using DATABASE_URL={}", url);

    // Try direct establish to capture underlying error
    match diesel::pg::PgConnection::establish(&url) {
        Ok(_) => println!("✅ Direct establish() succeeded."),
        Err(e) => {
            println!("❌ Direct establish() failed: {e}");
            // Continue to try pool to match the crate behavior
        }
    }

    // Initialize the global Diesel pool from DATABASE_URL
    // This uses the public re-export `init_db` from the `db` crate.
    match db::init_db() {
        Ok(_) => println!("✅ Pool initialized."),
        Err(e) => {
            println!("❌ Pool init failed: {e:?}");
            return Err(e);
        }
    }

    Ok(())
}
