use server::{server::Server, client::Client, handlers::input_handler};
use tokio::net::TcpListener;
use tokio::sync::mpsc::{self, *};
use tokio::io::AsyncReadExt;
//sync::mpsc::Receiver, thread};

fn nonblocking_stdin() -> UnboundedReceiver<String> {
    let (sender, receiver) = mpsc::unbounded_channel();

    std::thread::spawn(move || loop {
        let mut buf = String::new();
        std::io::stdin().read_line(&mut buf).unwrap();
        sender.send(buf).unwrap();
    });
    receiver
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> { 
    //let (sender, receiver) = mpsc::channel::<String>();
    let mut input = nonblocking_stdin();

    let port = std::env::args().nth(1).unwrap_or(String::from("1337"));
    let port = port.parse::<u32>().unwrap_or(1337);

    let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).await.unwrap();

    let server = Server::default();

    println!("Server started on port {}.", port);

    let clients = server.clients.clone();
    let mut server_clone = server.clone();
    tokio::spawn(async move {
        if let Some(data) = input.recv().await {
            input_handler(data, &mut server_clone).await;
        }
    });

    loop {
        if let Ok((stream, _)) = listener.accept().await {
            println!("New connection: {}", stream.peer_addr().unwrap());
            let (mut read, write) = stream.into_split();

            //let sender = sender.clone();
            let clients_clone = clients.clone();

            let mut server_clone = server.clone();
            tokio::spawn(async move {
                let mut buf = vec![0u8; 1024];

                let mut username = String::new();

                if let Err(e) = read.read(&mut buf).await {
                    println!("Failed to read from stream: {}", e.to_string());
                }

                buf.retain(|byte| byte != &u8::MIN);

                if !buf.is_empty() {
                    let message = String::from_utf8(buf).unwrap();
                    if message.contains("username") {
                        username = message.clone().split(' ').nth(1).unwrap().to_string();

                        let mut clients_lock = clients_clone.lock().await;

                        clients_lock.push(Client::new(write, username.clone()));

                        drop(clients_lock);
                    }
                }

                loop {
                    let mut buf = vec![0u8; 1024];

                    read.read(&mut buf).await.unwrap();

                    buf.retain(|byte| *byte != u8::MIN);

                    if !buf.is_empty() {
                        println!("{}", String::from_utf8(buf.clone()).unwrap());
                        server_clone.forward(String::from_utf8(buf).unwrap(), username.clone()).await;
                    }
                }

            });
        }

    }
}
