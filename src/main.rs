use core::task;
use once_cell::sync::Lazy;
use std::net::SocketAddr;
use std::os::unix::thread;
use std::sync::Arc;
use std::thread::spawn;
use tokio::main;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio::io::{self, AsyncBufReadExt, BufReader};

static CLIENTS: Lazy<Arc<Mutex<Vec<(TcpStream, SocketAddr)>>>> =
    Lazy::new(|| Arc::new(Mutex::new(Vec::new())));

async fn run_stream() {
    let tcp_listener = TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("Something went wrong");
    println!("TCP listening at 127.0.0.1:3000");

    loop {
        let (stream, addr) = tcp_listener.accept().await.unwrap();
        println!("New connection: {:?}", addr);

        let mut clients = CLIENTS.lock().await;
        clients.push((stream, addr));
    }
}

async fn check_clients_socket_connection_exists() {
    let clients = CLIENTS.lock().await;
    println!("-------------------------------------");
    for client in clients.iter() {
        match client.0.try_write(b"Hello, are you there?") {
            Ok(_) => println!("Client {:?} is alive", client.1),
            Err(_) => println!("Client {:?} might be disconnected", client.1),
        }
    }
    println!("-------------------------------------");

}

async fn remove_inactive_clients() {
    let mut clients = CLIENTS.lock().await; // Note: `mut` to allow modification
    println!("-------------------------------------");

    // Use retain to keep only alive clients
    clients.retain(|client| {
        match client.0.try_write(b"Hello, are you there?") {
            Ok(_) => {
                println!("Client {:?} is alive", client.1);
                true // Keep this client
            }
            Err(_) => {
                println!("Client {:?} is disconnected, removing...", client.1);
                false // Remove this client
            }
        }
    });

    println!("-------------------------------------");
}

async fn print_sockets() {
    let clients = CLIENTS.lock().await;
    println!("-------------------------------------");
    for client in clients.iter() {
        println!("{:?}", client.1);
    }
    println!("-------------------------------------");

}

#[main]
async fn main() {
    let stdin = BufReader::new(tokio::io::stdin());
    let mut lines = stdin.lines();

    let run_stream_handle = tokio::spawn(async {
        run_stream().await;
    });

    // Async command loop
    println!("Enter help to show commands: ");
    while let Some(input) = lines.next_line().await.unwrap() {
        let input = input.trim();
        if input == "0. print_socket" || input == "0" {
            print_sockets().await;
        } else if input == "1. check_clients" || input == "1" {
            check_clients_socket_connection_exists().await;
        }
        else if input == "2. remove_inactive_clients" || input == "2" {
            remove_inactive_clients().await;
        }
        else if input == "3. clear_logs" || input == "3" {
            print!("\x1B[2J\x1B[1;1H"); // ANSI escape code to clear the terminal
        }
        else if input == "help" || input == "h" {
            println!("-------------------------------------------------------------------");
            println!("nc can be used to dummy connect to the server using this command: `nc 127.0.0.1 3000`");
            println!("Available commands:");
            println!("0. print_socket             - Print all connected client sockets");
            println!("1. check_clients            - Check if clients are still connected");
            println!("2. remove_inactive_clients  - Remove disconnected clients");
            println!("3. clear_logs               - Clear the terminal logs");
            println!("h, help                     - Show this help message");
            println!("-------------------------------------------------------------------");

        }
        println!("Enter command: "); // Re-prompt

    }

    let _ = run_stream_handle.await;
}
