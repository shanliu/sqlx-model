mod common;
mod mysql;
#[tokio::main]
async fn main() {
    let mut args = std::env::args();
    let _ = args.next();
    let config = args.next().unwrap_or_else(|| "d2c_config.toml".to_string());
    let parse = common::ConfigParse::run(config.as_str()).await;
    if let Err(err) = parse {
        println!("error:{err}");
    }
}
