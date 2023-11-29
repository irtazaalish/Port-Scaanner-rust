use std::env;
use std::fs::File;
use std::io::{self, BufRead, Write};
use std::net::{IpAddr, SocketAddr};
use std::time::{Duration, Instant};
use tokio::net::TcpStream;
use tokio::time::timeout;
use std::sync::{Arc, Mutex};
use futures::future::try_join_all;

fn print_help() {
    println!("Syntax:");
    println!("  ./port_scanner <ip address> [options]");
    println!("Options:");
    println!("  -h               Show help");
    println!("  -p <1,2,3 or 10-30 or all> Specify ports (default: top 1024)");
    println!("  -t <threads>     Specify the number of threads (default: 4)");
    println!("  -f <file>        Specify a file containing IPs or hostnames");
    println!("  -o <output>      Specify an output file");
}

async fn scan_port(target: IpAddr, port: u16, output_file: &Option<String>) -> Result<(), ()> {
    let address: SocketAddr = format!("{}:{}", target, port)
        .parse()
        .expect("Failed to parse socket address");

    match timeout(Duration::from_secs(1), TcpStream::connect(address)).await {
        Ok(_) => {
            if let Some(output_file) = output_file {
                write_to_file(output_file, format!("{}: Port {} is open", target, port));
            } else {
                println!("{}: Port {} is open", target, port);
            }
            Ok(())
        }
        Err(_) => Err(()),
    }
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() <= 1 || args.contains(&String::from("-h")) {
        print_help();
        return;
    }

    let target = match args[1].parse() {
        Ok(ip) => ip,
        Err(_) => {
            eprintln!("Invalid IP address");
            return;
        }
    };

    let mut ports: Vec<u16> = (1..=1024).collect();
    let mut num_threads = 4;
    let mut output_file: Option<String> = None;

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "-p" => {
                if i + 1 < args.len() {
                    ports = parse_ports(&args[i + 1]);
                    i += 2;
                } else {
                    eprintln!("Invalid usage of -p flag");
                    print_help();
                    return;
                }
            }
            "-t" => {
                if i + 1 < args.len() {
                    num_threads = args[i + 1].parse().unwrap_or_else(|_| {
                        eprintln!("Invalid thread count");
                        print_help();
                        std::process::exit(1);
                    });
                    i += 2;
                } else {
                    eprintln!("Invalid usage of -t flag");
                    print_help();
                    return;
                }
            }
            "-f" => {
                if i + 1 < args.len() {
                    let file_path = &args[i + 1];
                    if let Err(_) = scan_from_file(file_path, &ports, num_threads, &output_file).await {
                        eprintln!("Error during port scanning from file");
                    }
                    return;
                } else {
                    eprintln!("Invalid usage of -f flag");
                    print_help();
                    return;
                }
            }
            "-o" => {
                if i + 1 < args.len() {
                    output_file = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Invalid usage of -o flag");
                    print_help();
                    return;
                }
            }
            _ => {
                eprintln!("Invalid flag: {}", args[i]);
                print_help();
                return;
            }
        }
    }

    println!("{}", "-".repeat(50));
    println!("Scanning Target: {:?}", target);
//    println!("Ports: {:?}", ports);
    println!("Threads: {:?}", num_threads);
    println!("Time Started: {:?}", Instant::now());

    let tasks = ports.into_iter().map(|port| scan_port(target, port, &output_file));

    let _results: Vec<_> = try_join_all(tasks).await.unwrap_or_else(|e| {
        eprintln!("Error during port scanning: {:?}", e);
        Vec::new()
    });

    println!("Time finished: {:?}", Instant::now());
}

fn parse_ports(ports_str: &str) -> Vec<u16> {
    let mut result = Vec::new();

    for part in ports_str.split(',') {
        if part == "all" {
            result.extend(1..=65535);
        } else if part.contains('-') {
            let range_parts: Vec<&str> = part.split('-').collect();
            if range_parts.len() == 2 {
                if let (Ok(start), Ok(end)) = (range_parts[0].parse::<u16>(), range_parts[1].parse()) {
                    result.extend(start..=end);
                }
            }
        } else {
            if let Ok(port) = part.parse() {
                result.push(port);
            }
        }
    }

    result
}

fn write_to_file(file_path: &str, content: String) {
    if let Ok(mut file) = File::create(file_path) {
        if let Err(err) = writeln!(file, "{}", content) {
            eprintln!("Error writing to file {}: {}", file_path, err);
        }
    } else {
        eprintln!("Error creating file: {}", file_path);
    }
}

async fn scan_from_file(file_path: &str, ports: &Vec<u16>, num_threads: usize, output_file: &Option<String>) -> Result<(), ()> {
    if let Ok(file) = File::open(file_path) {
        let reader = io::BufReader::new(file);
        let targets: Vec<String> = reader.lines().filter_map(|line| line.ok()).collect();

        let mut handles = vec![];

        for target in targets {
            let ports_clone = Arc::new(Mutex::new(ports.clone()));
            for _ in 0..num_threads {
                let ports_clone = Arc::clone(&ports_clone);
                let output_file_clone = output_file.clone();
                let target_clone = target.clone();
                let handle = tokio::spawn(async move {
                    while let Some(port) = {
                        let mut ports = ports_clone.lock().unwrap();
                        ports.pop()
                    } {
                        if let Err(_) = scan_port(target_clone.parse().unwrap(), port, &output_file_clone).await {
                            eprintln!("Error scanning port");
                        }
                    }
                });

                handles.push(handle);
            }
        }

        for handle in handles {
            if let Err(_) = handle.await {
                eprintln!("Error awaiting task");
            }
        }

        Ok(())
    } else {
        eprintln!("Error opening file: {}", file_path);
        Err(())
    }
}
