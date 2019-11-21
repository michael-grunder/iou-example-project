use std::{fs::File, io::IoSlice, os::unix::io::AsRawFd};

fn io_read_write(writes: u64, qd: u32) {
    let mut iou = iou::IoUring::new(qd).unwrap();

    let file = File::create("rust_testfile").unwrap();
    let fd = file.as_raw_fd();

    let slice = [IoSlice::new(b"012345678\n")];

    let mut nreads = 0;
    let mut nwrites = 0;
    let mut counter = 0;
    let mut bytesout = 0;
    let mut bytesin = 0;

    while nwrites < writes {
        let (op, obytes) = match iou.next_sqe() {
            Some(mut sqe) => unsafe {
                sqe.prep_write_vectored(fd, &slice, bytesout);
                bytesout += slice[0].len();
                nwrites += 1;
                counter += 1;
                ("WRITE", slice[0].len())
            },
            None => {
                let cqe = iou.wait_for_cqe().unwrap();
                let nbytes = cqe.result().unwrap();
                bytesin += nbytes;
                nreads += 1;
                counter -= 1;
                ("READ", nbytes)
            }
        };

        println!(
            "[{}] OP {:05} ({:03} bytes) | writes: {:02} ({:03} bytes) | reads: {:02} ({:03} bytes)",
            counter, op, obytes, nwrites, bytesout, nreads, bytesin
        );
    }

    println!("");
    while let Some(cqe) = iou.peek_for_cqe() {
        let obytes = cqe.result().unwrap();
        bytesin += obytes;
        nreads += 1;
        counter -= 1;
        println!(
            "[{}] OP {:05} ({:03} bytes) | writes: {:02} ({:03} bytes) | reads: {:02} ({:03} bytes)",
            counter, "READ", obytes, nwrites, bytesout, nreads, bytesin
        );
    }
}

fn main() {
    let writes = std::env::args()
        .nth(1)
        .unwrap_or("0".to_string())
        .parse::<u64>()
        .unwrap();

    let qd = std::env::args()
        .nth(2)
        .unwrap_or("0".to_string())
        .parse::<u32>()
        .unwrap();

    io_read_write(writes, qd);
}
