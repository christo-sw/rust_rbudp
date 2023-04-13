use std::{net::{TcpStream, TcpListener, UdpSocket}, io::{Read, Write}, sync::atomic::AtomicBool};
use std::{str, thread};

const PACKET_SIZE: usize = 512;

fn main() {
    let listener = TcpListener::bind("localhost:5050").unwrap();
    println!("Receiver listening on port {}", listener.local_addr().unwrap().port());

    let (stream, address) = listener.accept().unwrap();
    println!("Sender connected on address: {}", address);
    handle_tcp_flagging(stream);

}

fn handle_tcp_flagging(mut stream: TcpStream) {
    let mut received : [u8; 256] = [0; 256];
    stream.read(&mut received).expect("Error reading from sender");
    let print_str = str::from_utf8(&received).unwrap();

    println!("Message from sender: {}", print_str);

    let running = AtomicBool::new(true);

    // Start new thread for UDP receiving
    let handle = thread::spawn(move|| {
        handle_udp_receiving(&running);
    });

    // Wait for UDP thread to stop executing
    handle.join().unwrap();
}

fn handle_udp_receiving(running: &AtomicBool) {
    let udp_socket = UdpSocket::bind("localhost:5052").unwrap();
    let mut buf = [0 as u8; PACKET_SIZE];
    
    loop {
        udp_socket.recv(&mut buf).unwrap();
    }
}
