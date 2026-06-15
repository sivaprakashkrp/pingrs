use clap::Parser;
use colored::Colorize;
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr, ToSocketAddrs}, process::exit, thread::sleep, time::Duration
};

#[derive(Debug, Parser)]
#[command(
    version,
    author,
    about = "Ping utility written in RUST",
    long_about = "pingrs is a ping utility written in RUST with coloured output and statistics. The ping requests are sent at a time delay of 1 second from the previous.RTT and Packet size are recorded and display is easy to read output.",
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
        short = 'i',
        long = "infinite",
        help = "Pings infinitely"
    )]
    infinite: bool,
    #[arg(
        short = 'n',
        long = "n-times",
        help = "Number of times to ping"
    )]
    no_of_times: Option<u64>,
    #[arg(
        short = 'T',
        long = "ttl",
        help = format!("Time-To-Live for a ping request (Max: {})", u8::MAX)
    )]
    ttl: Option<u8>,
}

struct App {
    ip: IpAddr,
    port: u16,
    socket_addr: SocketAddr,
    target: String,
    timeout: u64,
    no_of_pings: u64,
    is_infinite: bool,
    ttl: u8,
}

fn main() {
    let cli = CLI::parse();

    // Global Application Parameters
    let mut app: App = App {
        ip: IpAddr::from(Ipv4Addr::UNSPECIFIED),
        port: 0,
        socket_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 7),
        target: format!("{}", cli.target),
        timeout: cli.timeout_secs.unwrap_or(5),
        no_of_pings: if cli.infinite {0} else {cli.no_of_times.unwrap_or(4)},
        is_infinite: cli.infinite,
        ttl: cli.ttl.unwrap_or(128),
    };

    // Resolving the IP address from Domain Name
    let domain_with_port = format!("{}:{}", cli.target, app.port);

    if let Ok(mut addrs) = domain_with_port.to_socket_addrs() {
        if let Some(sock_addr) = addrs.next() {
            app.socket_addr = sock_addr;
            app.ip = sock_addr.ip();
        }
        println!("{}: {}\n", "Resolved IP Address".blue().bold(), app.ip);
    } else {
        println!("{}", "Failed to resovle IP address of domain".red());
        exit(2);
    }

    // Preparing data for ICMP Packet
    let data = [1,2,3,4];  // ping data
    let timeout = Duration::from_secs(app.timeout);
    let options = ping_rs::PingOptions { ttl: app.ttl, dont_fragment: true };

    let mut count: u64 = 0;
    let mut pckts_sent: u64 = 0;
    let mut pckts_recv: u64 = 0;
    let mut pckts_lost: u64 = 0;
    let mut rtt_time_sum: u64 = 0;
    let mut max_rtt_time: u32 = 0;
    let mut min_rtt_time: u32 = u32::MAX;

    println!("{}", "No.\tBytes\tRTT\tTTL".blue().bold());

    while app.is_infinite || count < app.no_of_pings {
        let result = ping_rs::send_ping(&app.ip, timeout, &data, Some(&options));
        pckts_sent += 1;
        match result {
            Ok(reply) => {
                println!("{}.\t{}\t{}ms\t{}", count + 1, data.len(), reply.rtt, options.ttl);
                pckts_recv += 1;
                rtt_time_sum += reply.rtt as u64;
                if reply.rtt < min_rtt_time {min_rtt_time = reply.rtt};
                if reply.rtt > max_rtt_time {max_rtt_time = reply.rtt};
            },
            Err(e) => {
                println!("{}", format!("{:?}", e).red());
                pckts_lost += 1;
            }
        }
        count += 1;
        sleep(Duration::from_millis(1000));
    }

    println!("\n{}", "Ping Statistics:".blue().bold());
    println!("{}: {}\n{}: {}\n", "Target".yellow(), app.target, "IP Address".yellow(), app.ip);
    println!("{}:\nSent: {} | Received: {} | Lost: {} | Loss %: {}\n", "Packets".blue().bold(), format!("{}", pckts_sent).yellow().bold(), format!("{}", pckts_recv).green().bold(), format!("{}", pckts_lost).red().bold(), format!("{}%", ((pckts_lost / pckts_sent) * 100)).cyan().bold());
    println!("{}:\nMaximum: {} | Minimum: {} | Average: {}", "Round Trip Times (ms)".blue().bold(), format!("{} ms", max_rtt_time).red().bold(), format!("{} ms", min_rtt_time).green().bold(), format!("{} ms", rtt_time_sum / pckts_recv).cyan().bold());
}