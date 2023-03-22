use clap::Parser;
use env_logger::Env;

use tun2proxy::{main_entry, Proxy};

/// Tunnel interface to proxy
#[derive(Parser)]
#[command(author, version, about = "Tunnel interface to proxy.", long_about = None)]
struct Args {
    /// Name of the tun interface
    #[arg(short, long, value_name = "name", default_value = "tun0")]
    tun: String,

    /// The proxy URL in the form proto://[username[:password]@]host:port
    #[arg(short, long = "proxy", value_parser = Proxy::from_url, value_name = "URL")]
    proxy: Proxy,
}

fn main() {
    dotenv::dotenv().ok();
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let args = Args::parse();

    let addr = args.proxy.addr;
    let proxy_type = args.proxy.proxy_type;
    log::info!("Proxy {proxy_type} server: {addr}");

    main_entry(&args.tun, args.proxy);
}
