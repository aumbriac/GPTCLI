mod chat;
mod constants;
mod images;
mod utils;
mod vision;

use crate::utils::{print_help, process_command};
use std::{env, error::Error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv::dotenv().ok();
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 || args.iter().any(|arg| arg == "-help" || arg == "-h") {
        print_help();
        return if args.len() < 2 {
            Err("Insufficient arguments".into())
        } else {
            Ok(())
        };
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    process_command(&client, &args).await
}
