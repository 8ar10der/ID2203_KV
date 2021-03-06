use std::net::SocketAddr;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::{self, Receiver, Sender};

use omnipaxos_core::{
    ballot_leader_election::messages::BLEMessage, messages::Message,
    storage::memory_storage::MemoryStorage,
};

use omnipaxos_runtime::omnipaxos::{
    NodeConfig, OmniPaxosHandle, OmniPaxosNode, ReadEntry, ReadEntry::Decided,
    ReadEntry::Snapshotted,
};

use structopt::StructOpt;

mod models;
use crate::models::kv::{KVSnapshot, KeyValue};
use crate::models::msg::{CMDMessage, Msg, Operation};
use crate::models::node::Node;
use crate::models::package::{Package, Types};

mod configs;
use crate::configs::client::CLIENT_ADDR;
use crate::configs::server::DEBUG_OUTPUT;
use crate::configs::server::START_PORT;

#[tokio::main]
async fn main() {
    //get the args from terminal
    let node = Node::from_args();

    //create the node by args
    let mut node_conf = NodeConfig::default();
    node_conf.set_pid(node.pid);
    node_conf.set_peers(node.peers);

    //create a memory storage
    let storage = MemoryStorage::<KeyValue, KVSnapshot>::default();

    //create the omni paxos handler
    let OmniPaxosHandle {
        omni_paxos,
        seq_paxos_handle,
        ble_handle,
    } = OmniPaxosNode::new(node_conf, storage);

    //get the incoming and outgoing channel of BLE and SP
    let sp_in: mpsc::Sender<Message<KeyValue, KVSnapshot>> = seq_paxos_handle.incoming;
    let mut sp_out: mpsc::Receiver<Message<KeyValue, KVSnapshot>> = seq_paxos_handle.outgoing;
    let ble_in: mpsc::Sender<BLEMessage> = ble_handle.incoming;
    let mut ble_out: mpsc::Receiver<BLEMessage> = ble_handle.outgoing;

    //init the node address to string
    let port = START_PORT + node.pid;
    let addr = "127.0.0.1:".to_string() + &port.to_string();
    let addr: SocketAddr = addr.parse().unwrap();
    print_log(format!("Node address is {}", addr));

    //create three message channel for the communication between the treads later
    let (sp_sender, mut sp_rec) = mpsc::channel::<String>(24);
    let (ble_sender, mut ble_rec) = mpsc::channel::<String>(24);
    let (cmd_sender, mut cmd_rec) = mpsc::channel::<String>(24);

    //create the tasks
    let sp_out_task = sp_out_thread(&mut sp_out);
    let ble_out_task = ble_out_thread(&mut ble_out);
    let sp_in_task = sp_in_thread(&mut sp_rec, &sp_in);
    let ble_in_task = ble_in_thread(&mut ble_rec, &ble_in);
    let cmd_task = command_thread(&mut cmd_rec, &omni_paxos);
    let fw_task = forward_thread(&addr, &sp_sender, &ble_sender, &cmd_sender);

    //execute all tasks in parallel.
    tokio::join!(
        sp_out_task,
        ble_out_task,
        sp_in_task,
        ble_in_task,
        cmd_task,
        fw_task
    );
}

//The thread about the message forward
async fn forward_thread(
    addr: &SocketAddr,
    sp_sender: &Sender<String>,
    ble_sender: &Sender<String>,
    cmd_sender: &Sender<String>,
) {
    let tcp_listener = TcpListener::bind(addr).await.unwrap();
    loop {
        let sp_sender = sp_sender.clone();
        let ble_sender = ble_sender.clone();
        let cmd_sender = cmd_sender.clone();
        let (mut socket, _) = tcp_listener.accept().await.unwrap();

        tokio::spawn(async move {
            let (r, _) = socket.split();
            let mut reader = BufReader::new(r);
            let mut buffer = String::new();

            loop {
                print_log(format!("-----fw_thread-----"));
                let line = reader.read_line(&mut buffer).await.unwrap();
                if line == 0 {
                    break;
                }
                print_log(format!("receive string: {}", buffer));
                let pkg: Package = serde_json::from_str(&buffer).unwrap();
                print_log(format!("deserialized: {:?}", pkg));
                //send to corresponding thread
                match pkg.types {
                    Types::SP => {
                        //serialization
                        let msg = serde_json::to_string(&pkg.msg).unwrap();
                        sp_sender
                            .send(msg)
                            .await
                            .expect("Failed to send message to SP thread");
                    }
                    Types::BLE => {
                        //serialization
                        let msg = serde_json::to_string(&pkg.msg).unwrap();
                        ble_sender
                            .send(msg)
                            .await
                            .expect("Failed to send message to BLE thread");
                    }
                    Types::CMD => {
                        //serialization
                        let msg = serde_json::to_string(&pkg.msg).unwrap();
                        cmd_sender
                            .send(msg)
                            .await
                            .expect("Failed to send message to CMD thread");
                    }
                }
                buffer.clear();
            }
        });
    }
}

//SP messages outgoing thread
async fn sp_out_thread(sp_out: &mut mpsc::Receiver<Message<KeyValue, KVSnapshot>>) {
    loop {
        print_log(format!("-----sp_out_thread-----"));
        match sp_out.recv().await {
            Some(msg) => {
                print_log(format!("SP message: {:?} is received from channel", msg));
                let port = START_PORT + msg.to;
                let addr = "127.0.0.1:".to_string() + &port.to_string();
                let addr: SocketAddr = addr.parse().unwrap();

                let wrapped_msg = Package {
                    types: Types::SP,
                    msg: Msg::SP(msg),
                };

                let serialized = serde_json::to_string(&wrapped_msg).unwrap();

                if let Ok(mut tcp_stream) = TcpStream::connect(addr).await {
                    let (_, mut write) = tcp_stream.split();
                    write.write_all(serialized.as_bytes()).await.unwrap();
                }
            }
            None => {}
        }
    }
}

//SP messages incoming thread
async fn sp_in_thread(
    sp_rec: &mut Receiver<String>,
    sp_in: &mpsc::Sender<Message<KeyValue, KVSnapshot>>,
) {
    loop {
        print_log(format!("-----sp_in_thread-----"));
        match sp_rec.recv().await {
            Some(msg) => {
                let sp_msg = serde_json::from_str(&msg).unwrap();
                print_log(format!("SP message: {:?} is received from channel", sp_msg));
                sp_in
                    .send(sp_msg)
                    .await
                    .expect("Failed to send message to SP")
            }
            None => {}
        }
    }
}

//BLE messages outgoing thread
async fn ble_out_thread(ble_out: &mut mpsc::Receiver<BLEMessage>) {
    loop {
        print_log(format!("-----ble_out_thread-----"));
        match ble_out.recv().await {
            Some(msg) => {
                print_log(format!("BLE message: {:?} is received from channel", msg));
                let port = START_PORT + msg.to;
                let addr = "127.0.0.1:".to_string() + &port.to_string();
                let addr: SocketAddr = addr.parse().unwrap();

                let wrapped_msg = Package {
                    types: Types::BLE,
                    msg: Msg::BLE(msg),
                };

                let serialized = serde_json::to_string(&wrapped_msg).unwrap();

                if let Ok(mut tcp_stream) = TcpStream::connect(addr).await {
                    let (_, mut write) = tcp_stream.split();
                    write.write_all(serialized.as_bytes()).await.unwrap();
                }
            }
            None => {}
        }
    }
}

//BLE messages incoming thread
async fn ble_in_thread(ble_rec: &mut Receiver<String>, ble_in: &mpsc::Sender<BLEMessage>) {
    loop {
        print_log(format!("-----ble_in_thread-----"));
        match ble_rec.recv().await {
            Some(msg) => {
                print_log(format!("BLE message: {} is received from channel", msg));
                let sp_msg = serde_json::from_str(&msg).unwrap();
                ble_in
                    .send(sp_msg)
                    .await
                    .expect("Failed to send message to channel")
            }
            None => {}
        }
    }
}

//commands messages incoming thread
async fn command_thread(cmd_rec: &mut Receiver<String>, op: &OmniPaxosNode<KeyValue, KVSnapshot>) {
    loop {
        print_log(format!("-----cmd_thread-----"));
        match cmd_rec.recv().await {
            Some(msg) => {
                print_log(format!("Command: {} is received from network layer", msg));
                let msg: CMDMessage = serde_json::from_str(&msg).unwrap();
                match msg.operation {
                    Operation::Get => {
                        let key = msg.kv.key;
                        if let Some(entries) = op.read_entries(0..).await {
                            if let Some(v) = fetch_value(&key, entries.to_vec()).await {
                                let mut prefix = "This value is : ".to_string();
                                let value = v.to_string();
                                prefix += &value;
                                send_to_client(&prefix).await;
                            } else {
                                send_to_client("No value about the key").await;
                            }
                        } else {
                            //println!("This key does not exist");
                            send_to_client("No value about the key").await;
                        }
                    }

                    Operation::Put => {
                        //get the key value
                        let write_entry = msg.kv;
                        //append
                        if let Ok(_) = op.append(write_entry).await {
                            send_to_client("Successfully to put value").await;
                        } else {
                            send_to_client("Failed to put").await;
                        }
                    }
                    Operation::Snap => {
                        //something will cause omni paxos wrong
                        if let Ok(_) = op.snapshot(None, false).await {
                            send_to_client("Successfully to make a snapshot").await;
                        } else {
                            send_to_client("Failed to snapshot").await;
                        }
                    }
                }
            }
            None => {}
        }
    }
}

//to send message to client
async fn send_to_client(str: &str) {
    if let Ok(mut tcp_stream) = TcpStream::connect(CLIENT_ADDR).await {
        let (_, mut write) = tcp_stream.split();
        write.write_all(str.as_bytes()).await.unwrap();
        // print_log(format!("Replay: {} is send to network layer", &str));
        println!("Replay: {} is send to network layer", &str);
    } else {
        print_log(format!("Network failure"));
    }
}

//fetch value by key
async fn fetch_value(key: &str, vec: Vec<ReadEntry<KeyValue, KVSnapshot>>) -> Option<u64> {
    print!("vec {:?}", vec);
    let mut index = vec.len() - 1;
    let mut value = None;
    let vec = vec.clone();
    loop {
        match vec.get(index).unwrap() {
            Decided(kv) => {
                println!("Now kv {:?}", kv);
                if kv.key == key {
                    value = Some(kv.value);
                    break;
                }
            }
            Snapshotted(snapshotted_entry) => {
                let hashmap = snapshotted_entry.snapshot.snapshotted.clone();
                println!("Now snapshot {:?}", hashmap);
                if let Some(v) = hashmap.get(key) {
                    value = Some(*v);
                }
            }
            _ => {}
        }
        if index == 0 || value != None {
            break;
        }
        index -= 1;
    }
    value
}

//print logs into the terminal
fn print_log(log: String) {
    if DEBUG_OUTPUT {
        println!(" ");
        println!("{}", log);
        println!(" ");
    }
}
