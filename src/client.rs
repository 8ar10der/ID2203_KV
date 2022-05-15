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
use crate::configs::server::START_PORT;
use crate::models::kv::*;
use crate::models::msg::*;
use crate::models::package::*;

#[tokio::main]
async fn main() {
    //create client tasks
    let listener_task = listen_thread();
    let commander_task = command_thread();

    //execute all tasks in parallel.
    tokio::join!(listener_task, commander_task);
}

// the listener thread, receive message from server and print it
async fn listen_thread() {
    tokio::spawn(async move {
        let tcp_listener = TcpListener::bind(CLIENT_ADDR).await.unwrap();
        loop {
            let (mut socket, _) = tcp_listener.accept().await.unwrap();
            let (read, _) = socket.split();
            let mut reader = BufReader::new(read);
            let mut buffer = String::new();
            loop {
                let line = reader.read_line(&mut buffer).await.unwrap();
                if line == 0 {
                    break;
                }
                println!("Server: {}", &&buffer);
            }
        }
    });
}

// the interaction thread, get user's command and send it to the node
async fn command_thread() {
    loop {
        //choose the node
        println!("---------------------------");
        println!("Enter the pid of the node [eg. 2]:");
        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .expect("Stdin Read Error!");

        //create node's address
        let p: u64 = input.trim().parse().ok().expect("Parse Error");
        let mut addr = "127.0.0.1:".to_string();
        let port = START_PORT + p;
        addr += &port.to_string();
        let addr: SocketAddr = addr.parse().unwrap();

        //choose function
        loop {
            println!("---------------------------");
            println!("Please choose your command [input number 1/2/3]:");
            println!("1.Get");
            println!("2.Put");
            println!("3.Snap");

            let mut input = String::new();
            std::io::stdin().read_line(&mut input).expect("msg");

            //use match to run the sub function and get the
            //returned command message should be send
            let msg: CMDMessage = match input.trim() {
                "1" => get(),
                "2" => put(),
                "3" => snap(),
                _ => {
                    println!("Invalid command");
                    continue;
                }
            };

            //package message
            let pkg = Package {
                types: Types::CMD,
                msg: Msg::CMD(msg),
            };

            print_log(format!("{:?}", &pkg));

            //serialize to json string
            let bytes = serde_json::to_string(&pkg).unwrap();

            //create TCP stream
            if let Ok(mut tcp_stream) = TcpStream::connect(addr).await {
                let (_, mut write) = tcp_stream.split();
                write.write_all(bytes.as_bytes()).await.unwrap();
            } else {
                print!("Connection Error");
            }

            //wait reply from server
            thread::sleep(time::Duration::from_millis(10));
        }
    }
}

//get function
fn get() -> CMDMessage {
    println!("---------------------------");
    println!("Please enter the key [eg. A]:");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).expect("Get Error");
    CMDMessage {
        operation: Operation::Get,
        kv: KeyValue {
            key: input.trim().to_string(),
            value: 0,
        },
    }
}

//put function
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

//snap function
fn snap() -> CMDMessage {
    CMDMessage {
        operation: Operation::Snap,
        kv: KeyValue {
            key: String::from("_"),
            value: 0,
        },
    }
}

//log printer
fn print_log(log: String) {
    if DEBUG_OUTPUT {
        println!(" ");
        println!("{}", log);
        println!(" ");
    }
}
