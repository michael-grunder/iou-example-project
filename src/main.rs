use iou::{IoUring, SubmissionFlags};
use std::{
    collections::HashMap,
    convert::TryInto,
    fs::File,
    io::{BufRead, BufReader, IoSlice},
    os::unix::io::AsRawFd,
    pin::Pin,
};

struct WriteMarker<'a> {
    _bytes: Vec<u8>,
    iovec: [IoSlice<'a>; 1],
}

impl<'a> WriteMarker<'a> {
    fn new(buffer: Vec<u8>) -> Self {
        let ptr = unsafe { &*(buffer.as_ref() as *const _) };

        let container = Self {
            _bytes: buffer,
            iovec: [IoSlice::new(ptr)],
        };

        container
    }

    fn from_slice(slice: &[u8]) -> Self {
        Self::new(slice.to_vec())
    }

    fn pinned(slice: &[u8]) -> Pin<Box<Self>> {
        Box::pin(Self::from_slice(slice))
    }
}

fn main() {
    let infile = std::env::args().nth(1).expect("Pass an input file");
    let rdr = BufReader::new(File::open(infile).expect("Can't open file"));

    let file = File::create("rust_testfile").unwrap();
    let fd = file.as_raw_fd();
    let mut id = 0;
    let mut offset = 0;

    let mut iou = IoUring::new(1024).expect("Can't create ring");

    let mut map = HashMap::new();

    for mut line in rdr.lines().filter_map(|l| l.ok()) {
        line.push('\n');

        let io = WriteMarker::pinned(line.as_bytes());

        let mut sqe = iou.next_sqe().unwrap();
        sqe.set_user_data(id);
        unsafe {
            sqe.prep_write_vectored(fd, &io.iovec, offset);
            sqe.set_flags(SubmissionFlags::IO_LINK);
            offset += line.len();
        }

        map.insert(id, io);
        id += 1;
    }

    eprint!("Waiting for {} writes...", id);
    let _ = iou.wait_for_cqes(id.try_into().unwrap());
    eprintln!("done");
}
