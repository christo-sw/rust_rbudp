use std::net::{SocketAddr, UdpSocket};
use std::{
    io::{Read, Write},
    net::TcpStream,
};

mod file_handler;

const PACKET_SIZE: usize = 512;
const PACKET_NUM_SIZE: usize = 4;
const PACKET_DATA_SIZE: usize = PACKET_SIZE - PACKET_NUM_SIZE;
const WINDOW_SIZE: usize = 8; // in packets

fn main() {
    let address = "localhost:5050";
    println!("Connecting to receiver on {}", address);

    let stream = TcpStream::connect(address);
    match stream {
        Ok(stream) => rbudp_transfer(stream),
        Err(e) => println!("Error connecting to receiver: {}", e),
    }
}

fn rbudp_transfer(mut stream: TcpStream) {
    // Get filename from path and send it to receiver
    let path = "data/test2.txt";
    let collection: Vec<&str> = path.split("/").collect();
    let filename = collection.last().unwrap().to_string();
    stream
        .write(filename.as_bytes())
        .expect("Error writing filename receiver");

    // Open file to send
    let mut reader = file_handler::get_file_reader(path);

    // Setup UDP sending
    let udp_socket = UdpSocket::bind("localhost:5051").unwrap();
    let target_address: SocketAddr = "127.0.0.1:5052".parse().unwrap();

    // Setup and instantiate UDP variables
    let mut packet = [0 as u8; PACKET_SIZE];
    let mut buf = [0 as u8; PACKET_DATA_SIZE];
    let mut count = 0;
    let mut packet_num: u32 = 1; // Starts with packet number 1
    let mut num_bytes_read = PACKET_DATA_SIZE;

    // Setup and instantiate sliding window variables
    let mut window_packets = [[0 as u8; PACKET_SIZE]; WINDOW_SIZE];
    let mut sent_packet_nums_bytes = [0 as u8; PACKET_NUM_SIZE*WINDOW_SIZE];
    let mut start = 0;
    let mut end = WINDOW_SIZE - 1;

    // Main loop
    // TODO: change to loop based on WINDOW_SIZE*PACKET_DATA_SIZE data left in file
    while num_bytes_read == PACKET_DATA_SIZE {
        // Read window's packets data, create the packets, and send
        for i in 0..WINDOW_SIZE {
            num_bytes_read = file_handler::read_buf_from_file(&mut reader, &mut buf);
            pack_packet(&mut packet, &packet_num, &buf);
            window_packets[i] = packet;
            udp_socket.send_to(&packet, target_address).unwrap();

            println!("DEBUG: Sent packet {}", packet_num);

            // Increase counts
            packet_num += 1;
            count += 1;
        }

        // Populate sent packets list as byte arrays
        for i in 0..WINDOW_SIZE {
            for j in 0..PACKET_NUM_SIZE {
                sent_packet_nums_bytes[i + j] = window_packets[i][j];
            }
        }

        // Send list of sent packets
        stream.write(&sent_packet_nums_bytes).unwrap();

        // Wait for tcp confirmation of received packets
        // while stream.read() != null
    }

    println!("DEBUG: sent a total of {} packets", count);

    // Signal end of transfer
    stream
        .write("stop".as_bytes())
        .expect("Error sending 'stop' message");
    println!("Sent stop signal");
}

fn pack_packet(packet: &mut [u8; PACKET_SIZE], packet_num: &u32, buf: &[u8; PACKET_DATA_SIZE]) {
    // Convert count to byte array for stuffing into packet
    let packet_num_bytes = packet_num.to_le_bytes();

    // Add packet number to packet
    for i in 0..PACKET_NUM_SIZE {
        packet[i] = packet_num_bytes[i];
    }

    // Add data to packet after packet numbers
    for i in PACKET_NUM_SIZE..PACKET_SIZE {
        packet[i] = buf[i - PACKET_NUM_SIZE];
    }
}
