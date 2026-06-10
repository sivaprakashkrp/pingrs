use std::net::{IpAddr, Ipv4Addr, ToSocketAddrs};
use clap::Parser;

#[derive(Debug, Parser)]
#[command(
    version, author, 
    about = "Ping utility written in RUST",
    long_about = "Long about comes here",
    help_template = "{bin} {version}\nDeveloped By: {author}\n\n{about}\n\nUsage:\n\t{usage}\n\n{all-args}",
    author = "Sivaprakash P"
)]
struct CLI {
    domain: String
}

struct App {
    domain: String,
    ip_addr: IpAddr
}

fn main() {
    let cli = CLI::parse();
    let mut app: App = App {domain: format!("{}", cli.domain), ip_addr: IpAddr::V4(Ipv4Addr::UNSPECIFIED)};

    // Resolving the IP address from Domain Name
    let domain_with_port = format!("{}:0", cli.domain);

    if let Ok(mut addrs) = domain_with_port.to_socket_addrs() {
        if let Some(sock_addr) = addrs.next() {
            let ip = sock_addr.ip();
            app.ip_addr = ip;
            println!("IP address is: {}", ip);
        }
    } else {
        println!("Failed to resovle IP address of domain");
    }

    // Creating a socket for transferring ICMP packets
    
}

// To calculate checksum according to Internet Standards (RFC 1071)
