use iou::IoUring;
use std::{
    collections::HashMap,
    convert::TryInto,
    fs::File,
    io::{BufRead, BufReader, IoSlice},
    os::unix::io::AsRawFd,
    pin::Pin,
};

struct IouMarker<'a> {
    _data: Vec<Vec<u8>>,
    iovecs: Vec<IoSlice<'a>>,
    byte_len: usize,
}

impl<'a> IouMarker<'a> {
    // This also works but appears unneccessary since data_ref points to the
    // heap allocations.
    fn pinned<T>(writes: T) -> Pin<Box<Self>>
    where
        T: Into<Vec<Vec<u8>>>,
    {
        let writes = writes.into();
        let mut byte_len = 0;

        let iovecs: Vec<_> = writes
            .iter()
            .map(|b| {
                let ptr = unsafe { &*(b.as_ref() as *const _) };
                byte_len += b.len();
                IoSlice::new(ptr)
            })
            .collect();

        let container = Self {
            _data: writes,
            byte_len,
            iovecs,
        };

        Box::pin(container)
    }

    fn raw<T>(writes: T) -> Self
    where
        T: Into<Vec<Vec<u8>>>,
    {
        let writes = writes.into();
        let mut byte_len = 0;

        let iovecs: Vec<_> = writes
            .iter()
            .map(|b| {
                let ptr = unsafe { &*(b.as_ref() as *const _) };
                byte_len += b.len();
                IoSlice::new(ptr)
            })
            .collect();

        Self {
            _data: writes,
            byte_len,
            iovecs,
        }
    }

    fn len(&self) -> usize {
        self.iovecs.len()
    }

    fn byte_len(&self) -> usize {
        self.byte_len
    }
}

// Alternate use where we zip two files together line by line
fn zip_lines(f1: &str, f2: &str, qd: u32) {
    let rdr1 = BufReader::new(File::open(f1).expect("Can't open first file"));
    let rdr2 = BufReader::new(File::open(f2).expect("Can't open second file"));

    let zfile = File::create("rust_testzip").expect("Can't open zip output file");
    let fd = zfile.as_raw_fd();

    let mut iou = IoUring::new(qd).expect("Can't create ring");
    let mut map = HashMap::new();

    let mut inflight = 0;
    let mut offset = 0;
    let mut id = 0;

    for (l1, l2) in rdr1.lines().zip(rdr2.lines()) {
        if let (Ok(mut a), Ok(mut b)) = (l1, l2) {
            if inflight == qd {
                eprintln!("Pausing for {} inflight writes...", inflight);
                while inflight > 0 {
                    let _ = iou.wait_for_cqe();
                    inflight -= 1;
                }
            }

            a.push('\n');
            b.push('\n');

            let io = IouMarker::raw(vec![a.into_bytes(), b.into_bytes()]);

            let mut sqe = iou.next_sqe().unwrap();
            sqe.set_user_data(id);
            unsafe {
                sqe.prep_write_vectored(fd, &io.iovecs, offset);
                inflight += 1;
                offset += io.byte_len();
            }

            map.insert(id, io);
            id += 1;
        }
    }

    eprint!("Waiting for {} writes...", inflight);
    let _ = iou.wait_for_cqes(inflight.try_into().unwrap());
    eprintln!("done!");
}

fn iou_copy_lines(infile: &str, qd: u32) {
    let rdr = BufReader::new(File::open(infile).expect("Can't open input file"));

    let ofile = File::create("rust_testcopy").expect("Can't open output file");
    let fd = ofile.as_raw_fd();

    let mut iou = IoUring::new(qd).expect("Can't create ring");
    let mut map = HashMap::new();

    let mut inflight = 0;
    let mut offset = 0;
    let mut pauses = 0;
    let mut id = 0;

    for mut line in rdr.lines().filter_map(|l| l.ok()) {
        line.push('\n');

        if inflight == qd {
            while inflight > 0 {
                let cqe = iou.wait_for_cqe().unwrap();
                map.remove(&cqe.user_data());
                inflight -= 1;
                pauses += 1;
            }
        }

        let io = IouMarker::raw(vec![line.into_bytes()]);

        let mut sqe = iou.next_sqe().unwrap();
        sqe.set_user_data(id);
        unsafe {
            sqe.prep_write_vectored(fd, &io.iovecs, offset);
            inflight += 1;
            offset += io.byte_len();
        }

        map.insert(id, io);
        id += 1;

        if id % 1000 == 0 {
            eprintln!(
                "[{}]: In-flight: {}, pauses: {}, offset: {}",
                id, inflight, pauses, offset
            );
        }
    }

    eprint!("Waiting for final {} writes...", inflight);
    let _ = iou.wait_for_cqes(inflight.try_into().unwrap());
    eprintln!("done!");
}

fn main() {
    let args = std::env::args().collect::<Vec<_>>();

    let qd = if args.len() > 1 {
        args[1].parse().unwrap()
    } else {
        128
    };

    let f1 = if args.len() > 2 {
        &args[2]
    } else {
        "10k.lines"
    };

    // First change

    iou_copy_lines(f1, qd);
}
