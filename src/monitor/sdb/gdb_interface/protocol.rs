use std::io::{Read, Write};
use std::net::TcpStream;

#[allow(dead_code)]
pub fn hex_nibble(hex: u8) -> u8 {
    if hex.is_ascii_digit() { hex - '0' as u8 } else { hex.to_ascii_lowercase() - 'a' as u8 + 10 }
}

// fn hex_encode(digit: u8) -> u8 {
//     if digit > 9 { digit + 'a' as u8 - 10 } else { digit + '0' as u8 }
// }
#[allow(dead_code)]
pub fn gdb_decode_hex(msb: u8, lsb: u8) -> Option<u8> {
    if !msb.is_ascii_hexdigit() || !lsb.is_ascii_hexdigit() {
        return None;
    }
    Some(16 * hex_nibble(msb) + hex_nibble(lsb))
}

// fn gdb_decode_hex_str(bytes: &[u8]) -> u64 {
//     let mut val: u64 = 0;
//     let mut weight: u64 = 1;
//     for i in range_step(0, bytes.len(), 2) {
//         let res = gdb_decode_hex(bytes[i], bytes[i + 1]);
//         if res == None {
//             break;
//         }
//         val += weight * res.unwrap() as u64;
//         weight *= 16 * 16;
//     }
//     val
// }

fn read_checked(conn: &mut TcpStream, buf: &mut [u8]) {
    if conn.read(buf).unwrap() == 0 {
        panic!("qemu connection terminated")
    }
}

fn read_until_ack(conn: &mut TcpStream) -> bool {
    let mut buf = [0u8];
    loop {
        read_checked(conn, &mut buf);
        match buf[0] as char {
            '+' => return true,
            '-' => return false,
            _ => {}
        }
    }
}

pub fn send_packet(conn: &mut TcpStream, command: &str) {
    let mut sum: u8 = 0;
    for c in command.chars() {
        sum = sum.wrapping_add(c as u8);
    }
    let msg = format!("${}#{:02X}", command, sum);
    loop {
        conn.write_all(msg.as_bytes()).unwrap();
        conn.flush().unwrap();
        if read_until_ack(conn) {
            break;
        }
    }
}

pub fn recv_packet(conn: &mut TcpStream) -> Vec<u8> {
// pub fn recv_packet(conn: &mut BufReader<TcpStream>, conn_write: &mut TcpStream) -> Vec<u8> {
    // let mut buf = vec!();
    // let mut checksum_buf = [0u8; 2];
    // loop {
    //     let mut res = vec!();
    //     buf.clear();
    //     conn.read_until(b'$', &mut buf).unwrap();
    //     let size = conn.read_until(b'#', &mut res).unwrap();
    //     res.truncate(size - 1);
    //     let mut sum = 0u8;
    //     for i in &res {
    //         sum += i;
    //     }
    //     conn.read_exact(&mut checksum_buf).unwrap();
    //     let msb = checksum_buf[0];
    //     let lsb = checksum_buf[1];
    //     let checksum = gdb_decode_hex(msb, lsb);
    //     let ok = checksum.is_some_and(|cs| cs == sum);
    //     match ok {
    //         true => conn_write.write_all(b"+").unwrap(),
    //         false => conn_write.write_all(b"-").unwrap(),
    //     }
    //     conn_write.flush().unwrap();
    //     if !ok {
    //         return res;
    //     }
    // }


    let mut buf = [0u8];
    loop {
        let mut res = vec!();
        loop { // read until '$'
            read_checked(conn, &mut buf);
            if buf[0] as char == '$' {
                break;
            }
        }
        let mut sum = 0u8;
        loop { // read until '#'
            read_checked(conn, &mut buf);
            if buf[0] as char == '#' {
                break;
            }
            res.push(buf[0]);
            sum = sum.wrapping_add(buf[0]);
        }
        // checksum
        read_checked(conn, &mut buf);
        let msb = buf[0];
        read_checked(conn, &mut buf);
        let lsb = buf[0];
        let checksum = gdb_decode_hex(msb, lsb);
        let ok = checksum.is_some_and(|cs| cs == sum);
        // send ack/nack
        match ok {
            true => conn.write_all(b"+").unwrap(),
            false => conn.write_all(b"-").unwrap(),
        }
        conn.flush().unwrap();
        if ok {
            return res;
        }
    }
}