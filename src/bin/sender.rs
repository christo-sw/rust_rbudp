use std::io::{BufReader, ErrorKind, Read};
use std::net::{SocketAddr, UdpSocket};
use std::thread;
use std::{fs::File, io::Write, net::TcpStream};

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
    let filename = String::from("data/test1.txt");
    let file = match File::open(filename) {
        Ok(file) => file,
        Err(e) => panic!("Error opening file: {}", e),
    };
    let mut reader = BufReader::new(file);

    // Setup UDP sending
    //let udp_socket = UdpSocket::bind("localhost:5051").unwrap();
    let mut buf = [0 as u8; PACKET_SIZE];
    //let target_address: SocketAddr = "localhost:5051".parse().unwrap();

    while read_bytes_from_file(&mut reader, &mut buf) {
        //udp_socket.send_to(&buf, target_address).unwrap();
        dbg!(buf);
    }
}

fn read_bytes_from_file(reader: &mut BufReader<File>, buf: &mut [u8; PACKET_SIZE]) -> bool {
    // Reset buffer to avoid contamination at EOF read
    for i in 0..buf.len() {
        buf[i] = 0;
    }

    // Read from file to buffer
    match reader.read_exact(buf) {
        Ok(_) => return true,
        Err(_) => return false,
    }
}
