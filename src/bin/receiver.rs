use std::{
    io::{ErrorKind, Read, Write},
    net::{TcpListener, TcpStream, UdpSocket},
    time::Duration,
};

mod file_handler;

const PACKET_SIZE: usize = 512;
const PACKET_NUM_SIZE: usize = 4;
const PACKET_DATA_SIZE: usize = PACKET_SIZE - PACKET_NUM_SIZE;
const WINDOW_SIZE: usize = 8;

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
    stream
        .set_read_timeout(Some(Duration::from_millis(100)))
        .unwrap();
    let udp_socket = UdpSocket::bind("localhost:5052").unwrap();
    udp_socket
        .set_read_timeout(Some(Duration::from_millis(100)))
        .unwrap();

    // Setup and instantiate UDP variables
    let mut packet = [0 as u8; PACKET_SIZE];
    let mut buf = [0 as u8; PACKET_DATA_SIZE];
    let mut packet_num: u32 = 0;
    let mut count = 0;

    // Setup and instantiate sliding window variables
    let mut window_packets = [[0 as u8; PACKET_SIZE]; WINDOW_SIZE];

    // List of allegedly sent packet numbers for the current window as according to sender
    //let mut alleged_packet_nums = [0 as u32; WINDOW_SIZE];
    let mut alleged_packet_nums: Vec<u32> = Vec::new();
    let mut alleged_packet_nums_bytes = [0 as u8; PACKET_NUM_SIZE * WINDOW_SIZE];
    let mut missing_packet_nums: Vec<u32> = Vec::new();

    // List of actually received packet numbers for the current window
    let mut recvd_packet_nums: Vec<u32> = Vec::new();
    let mut start = 0;
    let mut end = WINDOW_SIZE - 1; // TODO: check if this should not be -1

    let mut running = true;

    // TODO: implement sliding window and packet backup
    // Receive data
    while running {
        // Try receiving a packet from the sender
        for i in 0..WINDOW_SIZE {
            match udp_socket.recv(&mut packet) {
                Ok(_) => {
                    // Get packet num and data (as buf)
                    unpack_packet(&packet, &mut packet_num, &mut buf);
                    recvd_packet_nums[i] = packet_num;

                    println!("DEBUG: Received packet {}", packet_num);

                    // Write the packet to file
                    // TODO: move this so that writing is in order
                    // file_handler::write_buf_to_file(&mut writer, &buf);

                    // Increase received packet count
                    count += 1;
                }
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                    continue;
                }
                Err(e) => panic!("Encountered IO Error: {}", e),
            };
        }
        // Check for tcp list of sent messages
        stream.read(&mut alleged_packet_nums_bytes).unwrap();

        // Get list of packet nums
        get_packet_nums(&mut alleged_packet_nums, &alleged_packet_nums_bytes);

        // Compare with list of actually received packet numbers
        if get_missing_packet_nums(
            &alleged_packet_nums,
            &mut recvd_packet_nums,
            &mut missing_packet_nums,
        ) {
            // Convert Vec<u32> to [u8; WINDOW_SIZE*PACKET_NUM_SIZE]
            let mut write_arr = [0 as u8; WINDOW_SIZE*PACKET_NUM_SIZE];
            let mut i = 0;
            for element in missing_packet_nums.iter() {
                let packet_num_bytes = (*element).to_le_bytes();
                for j in 0..PACKET_NUM_SIZE {
                    write_arr[i + j] = packet_num_bytes[j];
                }
                i += 1;
            }
            stream.write(&write_arr).unwrap();
            // TODO: finish
        }
    }

    println!("DEBUG: received a total of {} packets", count);

    //TODO: reopen file and trim to size (num bytes) sent by sender

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

fn get_packet_nums(
    alleged_packet_nums: &mut Vec<u32>,
    alleged_packet_nums_bytes: &[u8; PACKET_NUM_SIZE * WINDOW_SIZE],
) {
    for i in 0..WINDOW_SIZE {
        let mut packet_num_bytes = [0 as u8; PACKET_NUM_SIZE];
        for j in 0..PACKET_NUM_SIZE {
            packet_num_bytes[j] = alleged_packet_nums_bytes[i + j];
        }
        alleged_packet_nums[i] = u32::from_le_bytes(packet_num_bytes);
    }
}

/**
 * Compares a list of allegedly sent packet numbers with a list of actually received
 * packets, placing the differences in potentially disjoint places in the missing packets
 * array, with corresponding elements set to 0. Returns true if there were differences,
 * or false otherwise.
 */
fn get_missing_packet_nums(
    alleged_packet_nums: &Vec<u32>,
    recvd_packet_nums: &mut Vec<u32>,
    missing_packet_nums: &mut Vec<u32>,
) -> bool {
    let mut return_value = false;
    recvd_packet_nums.dedup();
    for element in alleged_packet_nums.iter() {
        if !recvd_packet_nums.contains(element) {
            return_value = true;
            missing_packet_nums.push(*element);
        }
    }
    return return_value;
}
