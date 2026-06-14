use clap::Parser;
// use pinger::{PingOptions, PingResult::Pong, ping};
use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, ToSocketAddrs, UdpSocket}, process::exit, str::from_utf8, time::{Duration, Instant}
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
    #[arg(
        short = 'T',
        long = "get-time",
        help = "Gets the time from server (Port 13)"
    )]
    get_time: bool,
}

struct App {
    ip: Ipv4Addr,
    port: u16,
    socket_addr: SocketAddr,
    target: String,
    timeout: Duration,
}

fn main() {
    let cli = CLI::parse();
    let mut app: App = App {
        ip: Ipv4Addr::UNSPECIFIED,
        // port: if cli.get_time {13} else {7},
        port: 0,
        socket_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 7),
        target: format!("{}", cli.target),
        timeout: Duration::from_secs(cli.timeout_secs.unwrap_or(5)),
    };

    // Resolving the IP address from Domain Name
    // Port 7 for Echo Protocol
    let domain_with_port = format!("{}:{}", cli.target, app.port);

    if let Ok(mut addrs) = domain_with_port.to_socket_addrs() {
        if let Some(sock_addr) = addrs.next() {
            let converted_addr = match sock_addr.ip() {
                IpAddr::V4(v4_addr) => Some(v4_addr),
                IpAddr::V6(v6_addr) => v6_addr.to_ipv4()
            };
            if let Some(ip) = converted_addr {
                app.socket_addr = sock_addr;
                app.ip = ip;
                println!("IP address is: {}", ip);
            } else {
                println!("Failed to resolve IPv4 address");
                exit(1);
            }
            
        }
    } else {
        println!("Failed to resovle IP address of domain");
        exit(2);
    }

    // let options = PingOptions::new(app.target, Duration::from_secs(app.timeout_secs), None);
    // match ping(options) {
    //     Ok(stream) => {
    //         for msg in stream {
    //             match msg {
    //                 Pong(duration, op_str) => {
    //                     println!("Duration: {:?}\tOutput String: {}", duration, op_str);
    //                 },
    //                 _ => {}
    //             }
    //         }
    //     },
    //     Err(err) => {
    //         println!("Error in pinging the source, Please try again:\n{}", err);
    //         exit(1);
    //     }
    // }

    // Ping function
    if let Err(err) = send_and_recv(&app) {
        println!("{}", err);
    }
}

fn send_and_recv(app: &App) -> Result<(), String> {
    match UdpSocket::bind("0.0.0.0:0") {
        Ok(socket) => {
            if let Err(err) = socket.set_read_timeout(Some(app.timeout)) {
                return Err(format!("Error in setting read_timeout: {}", err));
            }
            
            // Recording start timestamp for RTT calc
            let start = Instant::now();

            // Sending the request packet
            if let Err(err) = socket.send_to(b"Hello from pingrs", app.socket_addr) {
                return Err(format!("Error in sending packets: {}", err));
            }

            // Getting response
            let mut buf = [0;1024];
            match socket.recv_from(&mut buf) {
                Ok((len, src)) => {
                    let rtt = start.elapsed();
                    let response = from_utf8(&buf[..len]).unwrap_or("");

                    println!("Response from {}: {}", src, response);
                    println!("RTT: {} ms", rtt.as_millis());
                },
                Err(err) => {
                    return Err(format!("Request timed out or failed: {}", err));
                }
            }
        },
        Err(err) => {
            return Err(format!("Error in creating UDP socket: {}", err));
        }
    }
    Ok(())
}