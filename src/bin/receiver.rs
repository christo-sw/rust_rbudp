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

fn main() {
    //let num: u32 = 65000;
    //let bytes = num.to_le_bytes();
    //let num_from_bytes = u32::from_le_bytes(bytes);

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
    let mut received: [u8; 256] = [0; 256];
    stream
        .read(&mut received)
        .expect("Error reading from sender");
    let print_str = str::from_utf8(&received).unwrap();

    println!("Message from sender: {}", print_str);

    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();

    // Start new thread for UDP receiving
    let handle = thread::spawn(move || {
        // TODO: will have to use channels
        handle_udp_receiving(&running_clone);
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

fn handle_udp_receiving(running: &Arc<AtomicBool>) {
    // Open file to write to
    let mut writer = file_handler::get_file_writer("output/test2out.txt");

    // Setup UDP receiving
    let udp_socket = UdpSocket::bind("localhost:5052").unwrap();
    let mut packet = [0 as u8; PACKET_SIZE];
    
    udp_socket
        .set_read_timeout(Some(Duration::from_millis(100)))
        .unwrap();

    let mut count = 0;
    // Receive data
    while running.load(Ordering::Relaxed) {
        // Try receiving a packet from the sender
        match udp_socket.recv(&mut packet) {
            Ok(_) => {
                println!("DEBUG: Received packet {}", count);

                // Write the packet to file
                // TODO: change to only write buf part of packet
                file_handler::write_buf_to_file(&mut writer, &packet);

                // Increase packet count
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
