use std::net::TcpStream;
use num::range_step;
use crate::monitor::sdb::gdb_interface::protocol::{gdb_decode_hex, recv_packet, send_packet};

mod protocol;

#[allow(dead_code)]
pub struct GdbContext {
    conn: TcpStream,
}

#[allow(dead_code)]
impl GdbContext {
    pub fn new() -> Self {
        let conn = TcpStream::connect("127.0.0.1:1234").unwrap();
        Self {
            conn
        }
    }

    fn send(&mut self, command: &str) {
        send_packet(&mut self.conn, command);
    }

    fn receive(&mut self) -> Vec<u8> {
        // recv_packet(&mut self.conn_read, &mut self.conn)
        recv_packet(&mut self.conn)
    }
    pub fn talk(&mut self, command: &str) {
        self.send(command);
        self.receive();
        // println!("{}", std::str::from_utf8(&res).unwrap())
    }

    pub fn breakpoint(&mut self, addr: u64) {
        self.talk(format!("Z0,{:x}, 4", addr).as_str());
    }

    pub fn rm_breakpoint(&mut self, addr: u64) {
        self.talk(format!("z0,{:x}, 4", addr).as_str());
    }

    pub fn cont(&mut self) {
        self.talk("vCont;c:p1.-1");
    }

    pub fn step(&mut self) {
        self.talk("vCont;s:p1.-1")
        // self.continue_to_addr(pc + 4);
        // pc + 4
    }

    pub fn continue_to_addr(&mut self, addr: u64) {
        self.breakpoint(addr);
        self.cont();
        self.rm_breakpoint(addr);
    }

    pub fn read_regs_64(&mut self) -> Vec<u64> {
        self.send("g");
        let raw = self.receive();
        let mut res = vec!();
        for i in range_step(0, raw.len(), 16) {
            let mut val = 0u64;
            for j in range_step(0, 16, 2) {
                let byte = gdb_decode_hex(raw[i + j], raw[i + j + 1]).unwrap();
                val |= (byte as u64) << (4 * j)
            }
            res.push(val)
        }
        res
    }
}

#[cfg(test)]
mod tests {
    // use num::range_step;
    use crate::monitor::sdb::gdb_interface::GdbContext;
    // use crate::monitor::sdb::gdb_interface::protocol::{gdb_decode_hex, hex_nibble};

    // #[test]
    // fn test() {
    // let raw = b"04900080000000000000008000000000";
    // let mut res = vec!();
    // for i in range_step(0, raw.len(), 16) {
    //     let mut val = 0u64;
    //     for j in range_step(0, 16, 2) {
    //         let byte = gdb_decode_hex(raw[i + j], raw[i + j + 1]).unwrap();
    //         val |= (byte as u64) << (4 * j)
    //     }
    //     res.push(val)
    // }
    // for i in res {
    //     println!("{:#x}", i);
    // }
    // }

    #[test]
    fn it_works() {
        let mut ctx = GdbContext::new();
        let pc = 0x80000000;
        ctx.continue_to_addr(pc);
        // pc = ctx.step(pc);
        // pc = ctx.step(pc);
        ctx.step();
        let regs = ctx.read_regs_64();
        for i in 0..regs.len() {
            println!("{} {:#x}", i, regs[i]);
        }

        // ctx.talk("Z0,1004,4");
        // ctx.talk("$vCont;c:p1.-1#0f");
        // ctx.talk("$z0,1004,4#fb");
        // ctx.talk("$g#67");
    }
}