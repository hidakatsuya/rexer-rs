use crate::config::Config;
use crate::error::Result;

pub async fn init() -> Result<()> {
    let config = Config::new()?;

    if config.extensions_file_path().exists() {
        println!("{} already exists", config.extensions_file_path().display());
        return Ok(());
    }

    config.create_initial_config()?;
    println!("Created {}", config.extensions_file_path().display());
    Ok(())
}
