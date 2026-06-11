use clap::Parser;
use pinger::{PingOptions, PingResult::Pong, ping};
use std::{
    net::{IpAddr, Ipv4Addr, ToSocketAddrs},
    process::exit,
    time::Duration,
};

#[derive(Debug, Parser)]
#[command(
    version,
    author,
    about = "Ping utility written in RUST",
    long_about = "Long about comes here",
    help_template = "{bin} {version}\nDeveloped By: {author}\n\n{about}\n\nUsage:\n\t{usage}\n\n{all-args}",
    author = "Sivaprakash P"
)]
struct CLI {
    target: String,
    #[arg(
        short = 't',
        long = "timeout",
        help = "Sets the timeout for the packets (in seconds)"
    )]
    timeout_secs: Option<u64>,
}

struct App {
    ip: IpAddr,
    target: String,
    timeout_secs: u64,
}

fn main() {
    let cli = CLI::parse();
    let mut app: App = App {
        ip: IpAddr::from(Ipv4Addr::UNSPECIFIED),
        target: format!("{}", cli.target),
        timeout_secs: cli.timeout_secs.unwrap_or(2),
    };

    // Resolving the IP address from Domain Name
    let domain_with_port = format!("{}:0", cli.target);

    if let Ok(mut addrs) = domain_with_port.to_socket_addrs() {
        if let Some(sock_addr) = addrs.next() {
            let ip = sock_addr.ip();
            app.ip = ip;
            println!("IP address is: {}", ip);
        }
    } else {
        println!("Failed to resovle IP address of domain");
    }

    let options = PingOptions::new(app.target, Duration::from_secs(app.timeout_secs), None);
    match ping(options) {
        Ok(stream) => {
            for msg in stream {
                match msg {
                    Pong(duration, _) => {
                        println!("Duration: {:?}", duration);
                    },
                    _ => {}
                }
            }
        },
        Err(err) => {
            println!("Error in pinging the source, Please try again:\n{}", err);
            exit(1);
        }
    }

}