use std::time::Duration;
use std::{net::TcpStream, io::Write};
use std::thread;

fn main() {
    println!("Hello from sender!");

    let stream = TcpStream::connect("localhost:5050");
    match stream {
        Ok(stream) => handle_tcp_flagging(stream),
        Err(e) => println!("Error connecting to receiver: {}", e)
    }
}

fn handle_tcp_flagging(mut stream: TcpStream) {
    let greeting = "hello there!";
    stream.write(greeting.as_bytes()).expect("Error writing to receiver");

    // Create a new thread to handle UDP sending
    let handle = thread::spawn(|| {
        handle_udp_sending();
    });


    // Wait for UDP thread to stop executing
    println!("Wating for UDP thread to finish up...");
    handle.join().unwrap();
}

fn handle_udp_sending() {
    for i in 1..10 {
        println!("Hello {} from sender UDP thread!", i);
        thread::sleep(Duration::from_secs(1));
    }
}
