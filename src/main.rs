use iou::{IoUring, SubmissionFlags, SubmissionQueueEvent};
use std::{
    collections::HashMap,
    convert::TryInto,
    fs::File,
    io::{BufRead, BufReader, IoSlice},
    os::unix::io::AsRawFd,
    pin::Pin,
};

struct IoMarker<'a> {
    _bytes: Vec<u8>,
    iovec: [IoSlice<'a>; 1],
}

impl<'a> IoMarker<'a> {
    fn new(buffer: Vec<u8>) -> Pin<Box<Self>> {
        let ptr = unsafe { &*(buffer.as_ref() as *const _) };

        let container = Self {
            _bytes: buffer,
            iovec: [IoSlice::new(ptr)],
        };

        Box::pin(container)
    }

    fn len(&self) -> usize {
        self.iovec[0].len()
    }
}

fn main() {
    let infile = std::env::args().nth(1).expect("Pass an input file");
    let qd = std::env::args()
        .nth(2)
        .unwrap_or("127".to_string())
        .parse()
        .unwrap();

    let rdr = BufReader::new(File::open(infile).expect("Can't open file"));

    let file = File::create("rust_testfile").unwrap();
    let fd = file.as_raw_fd();
    let mut inflight = 0;
    let mut id = 0;
    let mut offset = 0;

    println!("Queue depth: {}", qd);

    let mut iou = IoUring::new(qd).expect("Can't create ring");

    let mut map = HashMap::new();

    for mut line in rdr.lines().filter_map(|l| l.ok()) {
        if inflight == qd {
            eprintln!("Pausing for {} inflight writes...", inflight);
            while inflight > 0 {
                let _ = iou.wait_for_cqe();
                inflight -= 1;
            }
        }

        line.push('\n');

        let io = IoMarker::new(line.into_bytes());

        let mut sqe = iou.next_sqe().unwrap();
        sqe.set_user_data(id);
        unsafe {
            sqe.prep_write_vectored(fd, &io.iovec, offset);
            sqe.set_flags(SubmissionFlags::IO_LINK);
            inflight += 1;
            offset += io.len();
        }

        map.insert(id, io);
        id += 1;
    }

    eprint!("Waiting for {} writes...", id);
    let _ = iou.wait_for_cqes(inflight.try_into().unwrap());
    eprintln!("done");
}
