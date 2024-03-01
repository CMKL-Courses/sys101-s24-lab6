use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, oneshot};

type Responder<T> = oneshot::Sender<Option<T>>;

#[derive(Debug)]
enum Command {
    Inc { resp: Responder<u8> },
    Dec { resp: Responder<u8> },
}

async fn read_buf(stream: &mut TcpStream) -> Option<u8> {
    let mut buffer = [0; 10];
    let n = stream.read(&mut buffer).await.unwrap();

    println!("Read to buffer {:?}", &buffer[..n]);
    if n > 0 {
        Some(buffer[0])
    } else {
        None
    }
}

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(32);
    let tx2 = tx.clone();

    // Spawn two tasks, one inc counter, another dec counter
    let t1 = tokio::spawn(async move {
        let (resp_tx, resp_rx) = oneshot::channel();
        let cmd = Command::Inc { resp: resp_tx };
        tx.send(cmd).await.unwrap();

        let res = resp_rx.await;
        println!("T1 Got {:?}", res);
    });

    let t2 = tokio::spawn(async move {
        let (resp_tx, resp_rx) = oneshot::channel();
        let cmd = Command::Dec { resp: resp_tx };
        tx2.send(cmd).await.unwrap();

        let res = resp_rx.await;
        println!("T2 Got {:?}", res);
    });

    let manager = tokio::spawn(async move {
        // Connect to a peer
        let mut client = TcpStream::connect("127.0.0.1:6379").await.unwrap();

        while let Some(cmd) = rx.recv().await {
            let result;
            match cmd {
                Command::Inc { resp } => {
                    client.write(&[0u8]).await.unwrap();
                    result = read_buf(&mut client).await;
                    resp.send(result).unwrap();
                    println!("Inc");
                },
                Command::Dec { resp } => {
                    client.write(&[1u8]).await.unwrap();
                    result = read_buf(&mut client).await;
                    resp.send(result).unwrap();
                    println!("Dec");
                },
            }
            println!("Got = {:?}", result);
        }
    });

    t1.await.unwrap();
    t2.await.unwrap();
    manager.await.unwrap();
}