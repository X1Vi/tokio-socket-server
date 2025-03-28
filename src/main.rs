use once_cell::sync::Lazy;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

static CLIENTS: Lazy<Arc<Mutex<Vec<(TcpStream, SocketAddr)>>>> =
    Lazy::new(|| Arc::new(Mutex::new(Vec::new())));

static CURRENT_CLIENT_SOCKET_ADDR: Lazy<Mutex<Option<SocketAddr>>> = Lazy::new(|| Mutex::new(None));

async fn set_current_client(index: usize) {
    let clients = CLIENTS.lock().await;
    if index < clients.len() {
        if let Some(client) = clients.get(index) {
            let mut current_client_addr = CURRENT_CLIENT_SOCKET_ADDR.lock().await;
            *current_client_addr = Some(client.1);
            println!("Selected client: {:?}", client.1);
        }
    } else {
        println!("Index out of bounds");
    }
}

async fn send_command_to_current_client(command: String) {
    let mut clients = CLIENTS.lock().await;
    for (tcpstream, socketaddr) in clients.iter_mut() {
    
        if *socketaddr == CURRENT_CLIENT_SOCKET_ADDR.lock().await.unwrap() {
                tcpstream.write(command.as_bytes()).await.unwrap();
        }
    }
}

async fn print_current_client_socket_addr() {
    let socketaddr = CURRENT_CLIENT_SOCKET_ADDR.lock().await.unwrap();
    println!("{:?}", socketaddr);
}

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
    for (index, client) in clients.iter().enumerate() {
        println!("{}: {:?}", index, client.1);
    }
    println!("-------------------------------------");
}

async fn broadcast_message(message: String) {
    let clients = CLIENTS.lock().await;
    println!("-------------------------------------");
    for client in clients.iter() {
        let socket_addr = client.1;
        match client.0.try_write(message.as_bytes()) {
            Ok(_) => println!("Message sent to {:?}", socket_addr),
            Err(e) => println!("Failed to send message to {:?}: {:?}", socket_addr, e),
        }
    }
    println!("-------------------------------------");
}

#[tokio::main]
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
        } else if input == "check_clients" || input == "1" {
            check_clients_socket_connection_exists().await;
        } else if input == "remove_inactive_clients" || input == "2" {
            remove_inactive_clients().await;
        } else if input == "clear_logs" || input == "3" {
            print!("\x1B[2J\x1B[1;1H"); // ANSI escape code to clear the terminal
        } else if input.starts_with("broadcast_message ") || input.starts_with("4 ") {
            if let Some(message) = input.strip_prefix("broadcast_message ") {
                broadcast_message(message.to_string()).await;
            } else if let Some(message) = input.strip_prefix("4 ") {
                broadcast_message(message.to_string()).await;
            }
        } else if input.starts_with("set_current_client ") || input.starts_with("5 ") {
            if let Some(message) = input.strip_prefix("set_current_client ") {
                if let Ok(index) = message.parse::<usize>() {
                    set_current_client(index).await;
                } else {
                    println!("Invalid index. Please enter a valid number.");
                }
            } else if let Some(message) = input.strip_prefix("5 ") {
                set_current_client(message.parse::<usize>().unwrap_or_else(|_| {
                    println!("Invalid index. Please enter a valid number.");
                    0 // Default to 0 if parsing fails
                }))
                .await;
            }
        }
        else if input.starts_with("send_command ") || input.starts_with("6 ") {
            if let Some(command) = input.strip_prefix("send_command ") {
            send_command_to_current_client(command.to_string()).await;
            } else if let Some(command) = input.strip_prefix("6 ") {
            send_command_to_current_client(command.to_string()).await;
            }
        }
        else if input == "print_current_index" || input == "7" {
            print_current_client_socket_addr().await;
        }
         else if input == "help" || input == "h" {
            println!("-------------------------------------------------------------------");
            println!(
                "nc can be used to dummy connect to the server using this command: `nc 127.0.0.1 3000`"
            );
            println!("Available commands:");
            println!("0. print_socket                   - Print all connected client sockets");
            println!("1. check_clients                  - Check if clients are still connected");
            println!("2. remove_inactive_clients        - Remove disconnected clients");
            println!("3. clear_logs                     - Clear the terminal logs");
            println!("4. broadcast_message <message>    - Broadcast a message to all clients");
            println!("5. set_current_client <index>     - Set the current client by index");
            println!("6. send_command <command>         - Send a command to the current client");
            println!("7. print_current_index            - Print the current client's socket address");
            println!("h, help                           - Show this help message");
            println!("-------------------------------------------------------------------");
        }
        println!("Enter command or h for help: "); // Re-prompt
    }

    let _ = run_stream_handle.await;
}