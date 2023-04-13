use std::net::{SocketAddr, UdpSocket};
use std::thread;
use std::{io::Write, net::TcpStream};

mod file_handler;

const PACKET_SIZE: usize = 512;

fn main() {
    println!("Hello from sender!");

    let stream = TcpStream::connect("localhost:5050");
    match stream {
        Ok(stream) => handle_tcp_flagging(stream),
        Err(e) => println!("Error connecting to receiver: {}", e),
    }
}

fn handle_tcp_flagging(mut stream: TcpStream) {
    let greeting = "hello there!";
    stream
        .write(greeting.as_bytes())
        .expect("Error writing to receiver");

    // Create a new thread to handle UDP sending
    let handle = thread::spawn(|| {
        handle_udp_sending();
    });

    // Wait for UDP thread to stop executing
    println!("Wating for UDP thread to finish up...");
    handle.join().unwrap();

    stream
        .write("stop".as_bytes())
        .expect("Error sending 'stop' message");
    println!("Sent stop signal");
}

fn handle_udp_sending() {
    // Open file to send
    let mut reader = file_handler::get_file_reader("data/test1.txt");

    // Setup UDP sending
    let udp_socket = UdpSocket::bind("localhost:5051").unwrap();
    let target_address: SocketAddr = "127.0.0.1:5052".parse().unwrap();

    let mut buf = [0 as u8; PACKET_SIZE];
    let mut amount = file_handler::read_buf_from_file(&mut reader, &mut buf);
    let mut count = 0;

    while amount == buf.len() {
        udp_socket.send_to(&buf, target_address).unwrap();
        println!("DEBUG: Sent packet {}", count);
        amount = file_handler::read_buf_from_file(&mut reader, &mut buf);
        count += 1;
    }

    // Send final few bytes left in buffer
    println!("DEBUG: Sent packet {}", count);
    udp_socket.send_to(&buf, target_address).unwrap();
}
