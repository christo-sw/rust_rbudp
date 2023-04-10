use std::{net::{TcpStream, TcpListener}, io::Read};
use std::{str, thread, time::Duration};

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

    // Start new thread for UDP receiving
    let handle = thread::spawn(|| {
        handle_udp_receiving();
    });

    // Wait for UDP thread to stop executing
    handle.join().unwrap();
}

fn handle_udp_receiving() {
    for i in 1..10 {
        println!("Hello {} from receiver UDP thread!", i);
        thread::sleep(Duration::from_secs(1));
    }
}
