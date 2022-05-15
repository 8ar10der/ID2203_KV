use std::net::SocketAddr;
use std::thread;
use std::time;
use tokio::io::AsyncBufReadExt;
use tokio::io::AsyncWriteExt;
use tokio::io::BufReader;
use tokio::net::TcpListener;
use tokio::net::TcpStream;

mod configs;
mod models;

use crate::configs::client::*;
use crate::configs::server::*;
use crate::models::kv::*;
use crate::models::msg::*;
use crate::models::package::*;

#[tokio::main]
async fn main() {
    let listener_task = listen_thread();
    let commander_task = command_thread();

    tokio::join!(listener_task, commander_task);
}

async fn listen_thread() {
    let tcp_listener = TcpListener::bind(CLIENT_ADDR).await.unwrap();

    let (mut socket, _) = tcp_listener.accept().await.unwrap();
    let (read, _) = socket.split();
    let mut reader = BufReader::new(read);
    let mut buffer = String::new();
    loop {
        let line = reader.read_line(&mut buffer).await.unwrap();
        //EOF
        if line == 0 {
            break;
        }
        println!("Server: {}", &&buffer);
    }
}

async fn command_thread() {
    loop {
        println!("---------------------------");
        println!("Enter the pid of the node [eg. 2]:");

        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .expect("Stdin Read Error!");
        let p: u64 = input.trim().parse().ok().expect("Parse Error");
        let mut addr = "127.0.0.1:".to_string();
        let port = START_PORT + p;
        addr += &port.to_string();
        let addr: SocketAddr = addr.parse().unwrap();

        loop {
            println!("---------------------------");
            println!("Please choose your command [input number 1/2/3]:");
            println!("1.Get");
            println!("2.Put");
            println!("3.Snap");

            let mut input = String::new();
            std::io::stdin().read_line(&mut input).expect("msg");
            let msg: CMDMessage = match input.trim() {
                "1" => get(),
                "2" => put(),
                "3" => snap(),
                _ => {
                    println!("Invalid command");
                    continue;
                }
            };

            let pkg = Package {
                types: Types::CMD,
                msg: Msg::CMD(msg),
            };

            let bytes = serde_json::to_string(&pkg).unwrap();

            if let Ok(mut tcp_stream) = TcpStream::connect(addr).await {
                let (_, mut write) = tcp_stream.split();
                write.write_all(bytes.as_bytes()).await.unwrap();
            } else {
                print!("Connection Error");
            }

            thread::sleep(time::Duration::from_millis(10));

            break;
        }
    }
}

fn get() -> CMDMessage {
    println!("---------------------------");
    println!("Please enter the key [eg. A]:");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).expect("Get Error");
    CMDMessage {
        operation: Operation::Get,
        kv: KeyValue {
            key: input.parse().ok().expect("Get Error"),
            value: 0,
        },
    }
}

fn put() -> CMDMessage {
    println!("---------------------------");
    println!("Please enter the key and value [eg. A 10]:");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).expect("Put Error");
    let kv: Vec<&str> = input.trim().split(" ").collect();
    CMDMessage {
        operation: Operation::Put,
        kv: KeyValue {
            key: kv[0].to_string(),
            value: kv[1].parse::<u64>().ok().expect("Error"),
        },
    }
}

fn snap() -> CMDMessage {
    println!("---------------------------");
    println!("Please enter the key and value [eg. A 10]:");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).expect("Snap Error");
    let kv: Vec<&str> = input.trim().split(" ").collect();
    CMDMessage {
        operation: Operation::Snap,
        kv: KeyValue {
            key: kv[0].to_string(),
            value: kv[1].parse::<u64>().ok().expect("Error"),
        },
    }
}
