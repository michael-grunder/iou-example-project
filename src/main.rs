use std::{fs::File, io::IoSlice, os::unix::io::AsRawFd};

fn io_read_write(writes: u64, qd: u32, use_peek: bool) {
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

    if use_peek {
        // This causes a SIGSEGV for me using iou master
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
    } else {
        // This blocks without reading the final write
        while nreads < nwrites {
            let cqe = iou.wait_for_cqe().unwrap();
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
}

fn get_arg<T>(n: usize, default: &str) -> T
where
    T: std::str::FromStr,
    <T as std::str::FromStr>::Err: std::fmt::Debug,
{
    std::env::args()
        .nth(n)
        .unwrap_or(default.to_string())
        .parse::<T>()
        .expect("Can't parse argument")
}

fn main() {
    let (writes, qd, use_peek): (u64, u32, u32) =
        (get_arg(1, "3"), get_arg(2, "2"), get_arg(3, "0"));

    io_read_write(writes, qd, use_peek != 0);
}
