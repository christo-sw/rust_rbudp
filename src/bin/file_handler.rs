use std::{
    fs::File,
    io::{BufReader, BufWriter, Read, Write},
};

/**
 * This method is for testing purposes only.
 */
fn main() {
    // Get reader from filename
    let mut reader = get_file_reader("data/test2.txt");
    let mut writer = get_file_writer("output/test1out.txt");
    let mut buf = [0 as u8; 512];

    // Test
    let mut count = 0;
    let mut wrote;
    let mut amount = read_buf_from_file(&mut reader, &mut buf);
    while amount == buf.len() {
        println!("read #{}", count);
        wrote = write_buf_to_file(&mut writer, &buf);
        println!("wrote {} bytes", wrote);
        count += 1;
        amount = read_buf_from_file(&mut reader, &mut buf);
    }

    // Write final few bytes left in buffer
    let ok = write_num_bytes_from_buf_to_file(&mut writer, &buf, amount);
    println!("final write returned {}", ok);
}

/**
 * Get a buffered reader from the specified file name.
 */
pub fn get_file_reader(filename: &str) -> BufReader<File> {
    let file = match File::open(filename) {
        Ok(file) => file,
        Err(e) => panic!("Error opening file: {}", e),
    };

    return BufReader::new(file);
}

/**
 * Get a buffered writer for the specified file name.
 */
pub fn get_file_writer(filename: &str) -> BufWriter<File> {
    let file = match File::create(filename) {
        Ok(file) => file,
        Err(e) => panic!("Error opening file: {}", e),
    };

    return BufWriter::new(file);
}

/**
 * Attempts to read PACKET_SIZE bytes from specified reader, returning true if a full
 * buffer was read, and false otherwise (implying that the file has reached its end).
 */
pub fn read_buf_from_file(reader: &mut BufReader<File>, buf: &mut [u8]) -> usize {
    // Reset buffer to avoid contamination at EOF read
    buf.fill(0);

    // Read from file to buffer
    match reader.read(buf) {
        Ok(amount) => return amount,
        Err(e) => panic!("Error reading from file: {}", e),
    }
}

/**
 * Attempts to write PACKET_SIZE bytes from buffer to the file, returning
 * the amount of bytes actually written.
 */
pub fn write_buf_to_file(writer: &mut BufWriter<File>, buf: &[u8]) -> usize {
    // Write from buffer to file
    let amount = match writer.write(buf) {
        Ok(amount) => amount,
        Err(e) => panic!("Error writing to file: {}", e),
    };

    return amount;
}

/**
 * Attempts to write num bytes from buffer to the file, returning true
 * if the write was successful, and false otherwise.
 */
pub fn write_num_bytes_from_buf_to_file(
    writer: &mut BufWriter<File>,
    buf: &[u8],
    num: usize,
) -> bool {
    let slice = &buf[0..num];
    match writer.write(slice) {
        Ok(_) => return true,
        Err(_) => return false,
    }
}
