use clap::Parser;
use socket2::{Domain, Protocol, Socket, Type};
use std::{
    mem::MaybeUninit,
    net::{Ipv4Addr, SocketAddr, ToSocketAddrs},
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
    domain: String,
    #[arg(
        short = 't',
        long = "timeout",
        help = "Sets the timeout for the packets (in seconds)"
    )]
    timeout_secs: Option<u64>,
}

struct App {
    domain: String,
    ip_addr: SocketAddr,
    timeout_secs: u64,
}

fn main() {
    let cli = CLI::parse();
    let mut app: App = App {
        domain: format!("{}", cli.domain),
        ip_addr: SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 0),
        timeout_secs: cli.timeout_secs.unwrap_or(2),
    };

    // Resolving the IP address from Domain Name
    let domain_with_port = format!("{}:0", cli.domain);

    if let Ok(mut addrs) = domain_with_port.to_socket_addrs() {
        if let Some(sock_addr) = addrs.next() {
            let ip = sock_addr.ip();
            app.ip_addr = sock_addr;
            println!("IP address is: {}", ip);
        }
    } else {
        println!("Failed to resovle IP address of domain");
    }

    // Creating a socket for transferring ICMP packets
    match Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::ICMPV4)) {
        Ok(socket) => {
            // Setting Timeouts
            if let Err(err) = socket.set_read_timeout(Some(Duration::from_secs(app.timeout_secs))) {
                println!("Error in setting read_timeout:\n {}", err);
                exit(1);
            }
            if let Err(err) = socket.set_write_timeout(Some(Duration::from_secs(app.timeout_secs)))
            {
                println!("Error in setting write_timeout:\n {}", err);
                exit(2);
            }

            // ICMP request echo packet
            let mut packet = vec![
                8, // Type of request -> Echo Request
                0, // Code
                0, 0, // Checksum -> Initialized to zero
                0, 1, // Identifier
                0, 1, // Sequence Number
            ];

            let checksum = calculate_checksum(&packet);
            packet[2] = (checksum >> 8) as u8;
            packet[3] = (checksum & 0xFF) as u8;

            if let Err(err) = socket.send_to(&packet, &app.ip_addr.into()) {
                println!("Error in sending data through socket:\n {}", err);
                exit(3);
            }
            println!("ICMP packet sent successfully!");

            // Receiving reply
            let mut reply: Vec<MaybeUninit<u8>> = vec![MaybeUninit::zeroed(); 256];
            if let Ok((bytes_received, responder)) = socket.recv_from(reply.as_mut_slice()) {
                println!(
                    "Received response from {} through {:?}",
                    app.ip_addr,
                    responder.as_socket_ipv4()
                );
            } else {
                println!("Error in capturing response or response not received back");
            }
        }
        Err(err) => {
            println!("Failed to create socket to destination:\n {}", err);
        }
    }
}

// To calculate checksum according to Internet Standards (RFC 1071)
fn calculate_checksum(buffer: &[u8]) -> u16 {
    let mut sum: u32 = 0u32;
    let mut chunks = buffer.chunks_exact(2);

    while let Some(chunk) = chunks.next() {
        sum += u32::from(u16::from_be_bytes([chunk[0], chunk[1]]));
    }

    if let Some(&remainder) = chunks.remainder().first() {
        sum += u32::from(u16::from_be_bytes([remainder, 0]));
    }

    while sum >> 16 > 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }

    return sum as u16;
}
