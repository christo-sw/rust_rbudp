use std::{
    io::{ErrorKind, Read},
    net::{TcpListener, TcpStream, UdpSocket},
    str,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

mod file_handler;

const PACKET_SIZE: usize = 512;
const PACKET_NUM_SIZE: usize = 4;

fn main() {
    let listener = TcpListener::bind("localhost:5050").unwrap();
    println!(
        "Receiver listening on port {}",
        listener.local_addr().unwrap().port()
    );

    let (stream, address) = listener.accept().unwrap();
    println!("Sender connected on address: {}", address);
    handle_tcp_flagging(stream);
}

fn handle_tcp_flagging(mut stream: TcpStream) {
    let mut filename_as_bytes: Vec<u8> = vec![]; 
    stream
        .read(&mut filename_as_bytes)
        .expect("Error reading filename from sender");


    let filename: String = String::from_utf8(filename_as_bytes).unwrap();
    println!("Filename: {}", filename);

    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();

    // Start new thread for UDP receiving
    let handle = thread::spawn(move || {
        handle_udp_receiving(&running_clone, filename);
    });

    let mut msg = String::new();
    stream
        .read_to_string(&mut msg)
        .expect("Error reading from sender");
    println!("DEBUG: Received message from sender: {}", msg);
    if msg.eq("stop") {
        println!("stopping");
        running.store(false, Ordering::Relaxed);
    }

    // Wait for UDP thread to stop executing
    handle.join().unwrap();
}

fn handle_udp_receiving(running: &Arc<AtomicBool>, filename: String) {
    // Open file to write to
    let collection: Vec<&str> = filename.split("/").collect();
    dbg!(collection);
    let mut writer = file_handler::get_file_writer("output/test2out.txt");

    // Setup UDP receiving
    let udp_socket = UdpSocket::bind("localhost:5052").unwrap();
    udp_socket
        .set_read_timeout(Some(Duration::from_millis(100)))
        .unwrap();

    // TODO: implement sliding window and packet backup

    let mut packet = [0 as u8; PACKET_SIZE];
    let mut packet_num: u32;
    let mut packet_num_as_bytes = [0 as u8; PACKET_NUM_SIZE];
    let mut buf = [0 as u8; PACKET_SIZE - PACKET_NUM_SIZE];
    let mut count = 0;

    // Receive data
    while running.load(Ordering::Relaxed) {
        // Try receiving a packet from the sender
        match udp_socket.recv(&mut packet) {
            Ok(_) => {
                // Get packet num and data (as buf)
                for i in 0..PACKET_NUM_SIZE {
                    packet_num_as_bytes[i] = packet[i];
                }

                for i in PACKET_NUM_SIZE..PACKET_SIZE {
                    buf[i - PACKET_NUM_SIZE] = packet[i];
                }

                packet_num = u32::from_le_bytes(packet_num_as_bytes);
                println!("DEBUG: Received packet {}", packet_num);

                // Write the packet to file
                file_handler::write_buf_to_file(&mut writer, &buf);

                // Increase received packet count
                count += 1;
            }
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                continue;
            }
            Err(e) => panic!("Encountered IO error: {e}"),
        };
    }

    // TODO: reopen file and remove trailing 0s
}
