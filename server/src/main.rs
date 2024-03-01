use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    let counter = Arc::new(Mutex::new(1u8));
    println!("Listening with counter {:?}", counter);

    loop {
        let (socket, _) = listener.accept().await.unwrap();

        // Clone the handle to the counter.
        let counter = counter.clone();
        println!("Accepted");
        tokio::spawn(async move {
            println!("{:?}", counter);
            process(socket, counter).await;
        });
    }
}

async fn process(mut p0: TcpStream, p1: Arc<Mutex<u8>>) {
    loop {
        let mut buffer = [ 0u8 ];
        let n = p0.read(&mut buffer).await.unwrap_or(0);
        if n > 0 {
            println!("Reading {} bytes from {:?}",n, p0);
            for i in 0..n {
                let write_buf;
                {
                    let mut p1_val = p1.lock().unwrap();
                    match buffer[i] {
                        0 => *p1_val += 1, // inc if read "0"
                        1 => *p1_val -= 1, // dec if read "1"
                        _ => break,
                    }
                    write_buf = [*p1_val];
                } // drop the MutexGuard
                p0.write_all(&write_buf).await.unwrap();
            }
        } else {
            // can not read anymore, terminate this task
            break;
        }
    }
}