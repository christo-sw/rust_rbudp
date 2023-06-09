use std::net::{SocketAddr, UdpSocket};
use std::thread;
use std::time::Duration;
use std::{io::Write, net::TcpStream};

mod file_handler;

const PACKET_SIZE: usize = 512;
const PACKET_NUM_SIZE: usize = 4;

fn main() {
    println!("Hello from sender!");

    let stream = TcpStream::connect("localhost:5050");
    match stream {
        Ok(stream) => handle_tcp_flagging(stream),
        Err(e) => println!("Error connecting to receiver: {}", e),
    }
}

fn handle_tcp_flagging(mut stream: TcpStream) {
    let filename = "data/test1.txt";
    let amount = stream
        .write(filename.as_bytes())
        .expect("Error writing filename receiver");
    println!("amount written {}", amount);

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

    let mut packet = [0 as u8; PACKET_SIZE];

    let mut buf = [0 as u8; PACKET_SIZE - PACKET_NUM_SIZE];
    let mut count: u32 = 0;
    let mut count_bytes = count.to_le_bytes();

    let mut amount = file_handler::read_buf_from_file(&mut reader, &mut buf);
    for i in 0..PACKET_NUM_SIZE {
        packet[i] = count_bytes[i];
    }

    for i in PACKET_NUM_SIZE..PACKET_SIZE {
        packet[i] = buf[i - PACKET_NUM_SIZE];
    }

    // TODO: comment this out to check if resending works for first packet
    thread::sleep(Duration::from_secs(1));

    while amount == buf.len() {
        udp_socket.send_to(&packet, target_address).unwrap();
        println!("DEBUG: Sent packet {}", count);

        count += 1;
        amount = file_handler::read_buf_from_file(&mut reader, &mut buf);

        count_bytes = count.to_le_bytes();

        for i in 0..PACKET_NUM_SIZE {
            packet[i] = count_bytes[i];
        }

        for i in PACKET_NUM_SIZE..PACKET_SIZE {
            packet[i] = buf[i - PACKET_NUM_SIZE];
        }

    }

    // Send final few bytes left in buffer
    println!("DEBUG: Sent packet {}", count);
    udp_socket.send_to(&packet, target_address).unwrap();
}
