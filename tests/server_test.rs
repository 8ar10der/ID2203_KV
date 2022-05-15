use core::time;
use std::thread;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

mod common;
use common::{
    kv::KeyValue,
    msg::{CMDMessage, Msg, Operation},
    package::{Package, Types},
};

const CLIENT: &str = "127.0.0.1:12345";

#[tokio::test]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    //lisent for incoming result from kv store
    let listener = TcpListener::bind(CLIENT).await.unwrap();

    //Test 1: Test that the node returns the error message correctly.
    {
        let message = CMDMessage {
            operation: Operation::Get,
            kv: KeyValue {
                key: String::from("key"),
                value: 0,
            },
        };
        let wrapped_msg = Package {
            types: Types::CMD,
            msg: Msg::CMD(message),
        };

        let serialized = serde_json::to_string(&wrapped_msg).unwrap();

        if let Ok(mut tcp_stream) = TcpStream::connect("127.0.0.1:11001").await {
            let (_, mut w) = tcp_stream.split();
            w.write_all(serialized.as_bytes()).await.unwrap();
        }
        let (mut socket, _) = listener.accept().await.unwrap();
        let (r, _) = socket.split();
        let mut reader = BufReader::new(r);
        let mut buf = String::new();
        reader.read_line(&mut buf).await.unwrap();

        assert_eq!(&buf, "No value about the key");
        buf.clear();
    }

    //Test2
    //Test if the node can store data correctly.
    {
        let message = CMDMessage {
            operation: Operation::Put,
            kv: KeyValue {
                key: String::from("key"),
                value: 0,
            },
        };

        let wrapped_msg = Package {
            types: Types::CMD,
            msg: Msg::CMD(message),
        };

        let serialized = serde_json::to_string(&wrapped_msg).unwrap();

        if let Ok(mut tcp_stream) = TcpStream::connect("127.0.0.1:11001").await {
            let (_, mut w) = tcp_stream.split();
            w.write_all(serialized.as_bytes()).await.unwrap();
        }
        let (mut socket, _) = listener.accept().await.unwrap();
        let (r, _) = socket.split();
        let mut reader = BufReader::new(r);
        let mut buf = String::new();
        reader.read_line(&mut buf).await.unwrap();

        assert_eq!(&buf, "Successfully to put value");
        buf.clear();
    }

    thread::sleep(time::Duration::from_millis(1000));

    //Test 3
    //Test that the node is returning data correctly.
    {
        let message = CMDMessage {
            operation: Operation::Get,
            kv: KeyValue {
                key: String::from("key"),
                value: 0,
            },
        };
        let wrapped_msg = Package {
            types: Types::CMD,
            msg: Msg::CMD(message),
        };
        let serialized = serde_json::to_string(&wrapped_msg).unwrap();
        if let Ok(mut tcp_stream) = TcpStream::connect("127.0.0.1:11001").await {
            let (_, mut w) = tcp_stream.split();
            w.write_all(serialized.as_bytes()).await.unwrap();
        }
        let (mut socket, _) = listener.accept().await.unwrap();
        let (r, _) = socket.split();
        let mut reader = BufReader::new(r);
        let mut buf = String::new();
        reader.read_line(&mut buf).await.unwrap();
        println!("{}", &buf);

        assert_eq!(&buf, "This value is : 0");
        buf.clear();
    }

    //Test4
    //Test if the correct data can be read on other nodes of the cluster.
    {
        let message = CMDMessage {
            operation: Operation::Get,
            kv: KeyValue {
                key: String::from("key"),
                value: 0,
            },
        };

        let wrapped_msg = Package {
            types: Types::CMD,
            msg: Msg::CMD(message),
        };

        let serialized = serde_json::to_string(&wrapped_msg).unwrap();

        if let Ok(mut tcp_stream) = TcpStream::connect("127.0.0.1:11002").await {
            let (_, mut w) = tcp_stream.split();
            w.write_all(serialized.as_bytes()).await.unwrap();
        }
        let (mut socket, _) = listener.accept().await.unwrap();
        let (r, _) = socket.split();
        let mut reader = BufReader::new(r);
        let mut buf = String::new();
        reader.read_line(&mut buf).await.unwrap();
        println!("{}", &buf);
        assert_eq!(&buf, "This value is : 0");
        buf.clear();
    }

    //Test5
    //Test if the data can be updated in the correct way
    {
        let message = CMDMessage {
            operation: Operation::Put,
            kv: KeyValue {
                key: String::from("key"),
                value: 1,
            },
        };
        let wrapped_msg = Package {
            types: Types::CMD,
            msg: Msg::CMD(message),
        };
        let serialized = serde_json::to_string(&wrapped_msg).unwrap();
        if let Ok(mut tcp_stream) = TcpStream::connect("127.0.0.1:11001").await {
            let (_, mut w) = tcp_stream.split();
            w.write_all(serialized.as_bytes()).await.unwrap();
        }
        let (mut socket, _) = listener.accept().await.unwrap();
        let (r, _) = socket.split();
        let mut reader = BufReader::new(r);
        let mut buf = String::new();
        reader.read_line(&mut buf).await.unwrap();
        println!("{}", &buf);
        assert_eq!(&buf, "Successfully to put value");
        buf.clear();
    }

    thread::sleep(time::Duration::from_millis(10000));

    //Test6
    //Test if the correct updated data can be read on another node
    let message = CMDMessage {
        operation: Operation::Get,
        kv: KeyValue {
            key: String::from("key"),
            value: 0,
        },
    };
    let wrapped_msg = Package {
        types: Types::CMD,
        msg: Msg::CMD(message),
    };
    let serialized = serde_json::to_string(&wrapped_msg).unwrap();
    if let Ok(mut tcp_stream) = TcpStream::connect("127.0.0.1:11001").await {
        let (_, mut w) = tcp_stream.split();
        w.write_all(serialized.as_bytes()).await.unwrap();
    }
    let (mut socket, _) = listener.accept().await.unwrap();
    let (r, _) = socket.split();
    let mut reader = BufReader::new(r);
    let mut buf = String::new();
    reader.read_line(&mut buf).await.unwrap();
    println!("{}", &buf);
    assert_eq!(&buf, "This value is : 1");
    buf.clear();

    thread::sleep(time::Duration::from_millis(10000));

    //test7 read the updated key-value from node 2
    let message = CMDMessage {
        operation: Operation::Get,
        kv: KeyValue {
            key: String::from("key"),
            value: 0,
        },
    };
    let wrapped_msg = Package {
        types: Types::CMD,
        msg: Msg::CMD(message),
    };
    let serialized = serde_json::to_string(&wrapped_msg).unwrap();
    //send
    if let Ok(mut tcp_stream) = TcpStream::connect("127.0.0.1:11002").await {
        let (_, mut w) = tcp_stream.split();
        w.write_all(serialized.as_bytes()).await.unwrap();
    }
    let (mut socket, _) = listener.accept().await.unwrap();
    let (r, _) = socket.split();
    let mut reader = BufReader::new(r);
    let mut buf = String::new();
    reader.read_line(&mut buf).await.unwrap();
    println!("{}", &buf);
    assert_eq!(&buf, "This value is : 1");
    buf.clear();

    //Test8
    //Snapshot Testing
    {
        let message = CMDMessage {
            operation: Operation::Snap,
            kv: KeyValue {
                key: String::from("snapshot"),
                value: 0,
            },
        };
        let wrapped_msg = Package {
            types: Types::CMD,
            msg: Msg::CMD(message),
        };
        let serialized = serde_json::to_string(&wrapped_msg).unwrap();
        if let Ok(mut tcp_stream) = TcpStream::connect("127.0.0.1:11001").await {
            let (_, mut w) = tcp_stream.split();
            w.write_all(serialized.as_bytes()).await.unwrap();
        }
        let (mut socket, _) = listener.accept().await.unwrap();
        let (r, _) = socket.split();
        let mut reader = BufReader::new(r);
        let mut buf = String::new();
        reader.read_line(&mut buf).await.unwrap();
        println!("{}", &buf);
        assert_eq!(&buf, "Successfully to make a snapshot");
        buf.clear();
    }

    //Test9
    //Read after snapshot
    {
        let message = CMDMessage {
            operation: Operation::Get,
            kv: KeyValue {
                key: String::from("key"),
                value: 0,
            },
        };
        let wrapped_msg = Package {
            types: Types::CMD,
            msg: Msg::CMD(message),
        };
        let serialized = serde_json::to_string(&wrapped_msg).unwrap();
        //send
        if let Ok(mut tcp_stream) = TcpStream::connect("127.0.0.1:11001").await {
            let (_, mut w) = tcp_stream.split();
            w.write_all(serialized.as_bytes()).await.unwrap();
        }
        let (mut socket, _) = listener.accept().await.unwrap();
        let (r, _) = socket.split();
        let mut reader = BufReader::new(r);
        let mut buf = String::new();
        reader.read_line(&mut buf).await.unwrap();
        println!("{}", &buf);
        assert_eq!(&buf, "This value is : 1");
        buf.clear();
    }

    //Test10
    //Update the key-value after snapshot
    {
        let message = CMDMessage {
            operation: Operation::Put,
            kv: KeyValue {
                key: String::from("key"),
                value: 2,
            },
        };
        let wrapped_msg = Package {
            types: Types::CMD,
            msg: Msg::CMD(message),
        };
        let serialized = serde_json::to_string(&wrapped_msg).unwrap();
        if let Ok(mut tcp_stream) = TcpStream::connect("127.0.0.1:11001").await {
            let (_, mut w) = tcp_stream.split();
            w.write_all(serialized.as_bytes()).await.unwrap();
        }
        let (mut socket, _) = listener.accept().await.unwrap();
        let (r, _) = socket.split();
        let mut reader = BufReader::new(r);
        let mut buf = String::new();
        reader.read_line(&mut buf).await.unwrap();
        println!("{}", &buf);

        assert_eq!(&buf, "Successfully to put value");
        buf.clear();
    }

    thread::sleep(time::Duration::from_millis(10000));

    //Test11
    //Read the updated key-value after snapshot
    {
        let message = CMDMessage {
            operation: Operation::Get,
            kv: KeyValue {
                key: String::from("key"),
                value: 0,
            },
        };
        let wrapped_msg = Package {
            types: Types::CMD,
            msg: Msg::CMD(message),
        };
        let serialized = serde_json::to_string(&wrapped_msg).unwrap();
        if let Ok(mut tcp_stream) = TcpStream::connect("127.0.0.1:11001").await {
            let (_, mut w) = tcp_stream.split();
            w.write_all(serialized.as_bytes()).await.unwrap();
        }
        let (mut socket, _) = listener.accept().await.unwrap();
        let (r, _) = socket.split();
        let mut reader = BufReader::new(r);
        let mut buf = String::new();
        reader.read_line(&mut buf).await.unwrap();
        println!("{}", &buf);

        assert_eq!(&buf, "This value is : 2");
        buf.clear();
    }
    Ok(())
}
