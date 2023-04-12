use std::net::{SocketAddr, UdpSocket};
use std::thread;
use std::{io::Write, net::TcpStream};

mod lib;

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
}

fn handle_udp_sending() {
    // Open file to send

    // Setup UDP sending
    //let udp_socket = UdpSocket::bind("localhost:5051").unwrap();
    let mut reader = get_file_reader("data/test2.txt");


    let mut buf = [0 as u8; PACKET_SIZE];
    //let target_address: SocketAddr = "localhost:5051".parse().unwrap();

   // while read_bytes_from_file(&mut reader, &mut buf) {
        //udp_socket.send_to(&buf, target_address).unwrap();
    //    dbg!(buf);
    //}
}

