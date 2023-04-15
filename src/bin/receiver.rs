use std::{
    io::{ErrorKind, Read},
    net::{TcpListener, TcpStream, UdpSocket},
    str,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

mod file_handler;

const PACKET_SIZE: usize = 512;
const PACKET_NUM_SIZE: usize = 4;
const PACKET_DATA_SIZE: usize = PACKET_SIZE - PACKET_NUM_SIZE;

fn main() {
    let listener = TcpListener::bind("localhost:5050").unwrap();
    println!(
        "Receiver listening on port {}",
        listener.local_addr().unwrap().port()
    );

    let (stream, address) = listener.accept().unwrap();
    println!("Sender connected on address: {}", address);
    handle_transfer(stream);
}

fn handle_transfer(mut stream: TcpStream) {
    let mut filename_as_bytes = [0 as u8; 256];
    let amount = stream
        .read(&mut filename_as_bytes)
        .expect("Error reading filename from sender");
    let filename_as_bytes_slice = &filename_as_bytes[0..amount];
    let filename: String = String::from_utf8(filename_as_bytes_slice.to_vec()).unwrap();

    // Open file to write to
    println!("DEBUG: filename is {}", filename);
    let mut writer = file_handler::get_file_writer(filename);

    // Setup UDP receiving
    let udp_socket = UdpSocket::bind("localhost:5052").unwrap();
    udp_socket
        .set_read_timeout(Some(Duration::from_millis(100)))
        .unwrap();

    // Setup and instantiate UDP variables
    let mut packet = [0 as u8; PACKET_SIZE];
    let mut buf = [0 as u8; PACKET_DATA_SIZE];
    let mut packet_num: u32 = 0;
    let mut count = 0;

    let mut running = true;

    // TODO: implement sliding window and packet backup
    // Receive data
    while running {
        // Try receiving a packet from the sender
        match udp_socket.recv(&mut packet) {
            Ok(_) => {
                // Get packet num and data (as buf)
                unpack_packet(&packet, &mut packet_num, &mut buf);
                println!("DEBUG: Received packet {}", packet_num);

                // Write the packet to file
                file_handler::write_buf_to_file(&mut writer, &buf);

                // Increase received packet count
                count += 1;
            }
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                continue;
            }
            Err(e) => panic!("Encountered IO Error: {}", e),
        };
    }

    println!("DEBUG: received a total of {} packets", count);

    // Check for incoming message
    let mut msg = String::new();
    stream
        .read_to_string(&mut msg)
        .expect("Error reading from sender");
    println!("DEBUG: Received message from sender: {}", msg);
    if msg.eq("stop") {
        println!("stopping");
    }
}

fn unpack_packet(
    packet: &[u8; PACKET_SIZE],
    packet_num: &mut u32,
    buf: &mut [u8; PACKET_DATA_SIZE],
) {
    let mut packet_num_as_bytes = [0 as u8; PACKET_NUM_SIZE];
    for i in 0..PACKET_NUM_SIZE {
        packet_num_as_bytes[i] = packet[i];
    }

    for i in PACKET_NUM_SIZE..PACKET_SIZE {
        buf[i - PACKET_NUM_SIZE] = packet[i];
    }

    *packet_num = u32::from_le_bytes(packet_num_as_bytes);
}
