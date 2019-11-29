use iou::{IoUring, SubmissionFlags};
use std::{
    collections::HashMap,
    convert::TryInto,
    fs::File,
    io::{BufRead, BufReader, IoSlice},
    marker::PhantomPinned,
    os::unix::io::AsRawFd,
    pin::Pin,
    ptr::NonNull,
};

struct IoMarker<'a> {
    _data: Vec<u8>,
    iovec: Vec<IoSlice<'a>>,
}

impl<'a> IoMarker<'a> {
    fn new(data: Vec<u8>) -> Self {
        let data_ref = unsafe { &*(data.as_ref() as *const _) };

        Self {
            _data: data,
            iovec: vec![IoSlice::new(data_ref)],
        }
    }

    fn byte_len(&self) -> usize {
        self._data.len()
    }
}

struct IouMarker<'a> {
    _data: Vec<Vec<u8>>,
    iovecs: Vec<IoSlice<'a>>,
    byte_len: usize,
}

impl<'a> IouMarker<'a> {
    fn new<T>(writes: T) -> Pin<Box<Self>>
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

    fn len(&self) -> usize {
        self.iovecs.len()
    }

    fn byte_len(&self) -> usize {
        self.byte_len
    }
}

//fn zip2(f1: &str, f2: &str, qd: u32) {
//    let rdr1 = BufReader::new(File::open(f1).expect("Can't open first file"));
//    let rdr2 = BufReader::new(File::open(f2).expect("Can't open second file"));
//
//    let zfile = File::create("rust_testzip").expect("Can't open zip output file");
//    let fd = zfile.as_raw_fd();
//
//    let mut iou = IoUring::new(qd).expect("Can't create ring");
//    let mut map = HashMap::new();
//
//    let mut inflight = 0;
//    let mut offset = 0;
//    let mut id = 0;
//
//    for (l1, l2) in rdr1.lines().zip(rdr2.lines()) {
//        if let (Ok(mut a), Ok(mut b)) = (l1, l2) {
//            if inflight == qd - 1 {
//                eprintln!("Pausing for {} inflight writes...", inflight);
//                while inflight > 0 {
//                    let cqe = iou.wait_for_cqe().unwrap();
//                    map.remove(&cqe.user_data());
//                    inflight -= 1;
//                }
//            }
//
//            a.push('\n');
//            b.push('\n');
//
//            let ios = vec![IoMarker::new(a.into_bytes()), IoMarker::new(b.into_bytes())];
//
//            for io in ios.into_iter() {
//                let mut sqe = iou.next_sqe().unwrap();
//                sqe.set_user_data(id);
//                unsafe {
//                    sqe.prep_write_vectored(fd, &io.iovec, offset);
//                    inflight += 1;
//                    offset += io.byte_len();
//                }
//
//                map.insert(id, io);
//                id += 1;
//            }
//        }
//    }
//
//    eprint!("Waiting for {} writes...", inflight);
//    let _ = iou.wait_for_cqes(inflight.try_into().unwrap());
//    eprintln!("done!");
//}
//
//fn zip(f1: &str, f2: &str, qd: u32) {
//    let rdr1 = BufReader::new(File::open(f1).expect("Can't open first file"));
//    let rdr2 = BufReader::new(File::open(f2).expect("Can't open second file"));
//
//    let zfile = File::create("rust_testzip").expect("Can't open zip output file");
//    let fd = zfile.as_raw_fd();
//
//    let mut iou = IoUring::new(qd).expect("Can't create ring");
//    let mut map = HashMap::new();
//
//    let mut inflight = 0;
//    let mut offset = 0;
//    let mut id = 0;
//
//    for (l1, l2) in rdr1.lines().zip(rdr2.lines()) {
//        if let (Ok(mut a), Ok(mut b)) = (l1, l2) {
//            if inflight == qd {
//                eprintln!("Pausing for {} inflight writes...", inflight);
//                while inflight > 0 {
//                    let _ = iou.wait_for_cqe();
//                    inflight -= 1;
//                }
//            }
//
//            a.push('\n');
//            b.push('\n');
//
//            let io = IouMarker::new(vec![a.into_bytes(), b.into_bytes()]);
//
//            let mut sqe = iou.next_sqe().unwrap();
//            sqe.set_user_data(id);
//            unsafe {
//                sqe.prep_write_vectored(fd, &io.iovecs, offset);
//                inflight += 1;
//                offset += io.byte_len();
//            }
//
//            map.insert(id, io);
//            id += 1;
//        }
//    }
//
//    eprint!("Waiting for {} writes...", inflight);
//    let _ = iou.wait_for_cqes(inflight.try_into().unwrap());
//    eprintln!("done!");
//}

fn copy(infile: &str, qd: u32) {
    let rdr = BufReader::new(File::open(infile).expect("Can't open input file"));

    let ofile = File::create("rust_testcopy").expect("Can't open output file");
    let fd = ofile.as_raw_fd();

    let mut iou = IoUring::new(qd).expect("Can't create ring");
    let mut map = HashMap::new();

    let mut inflight = 0;
    let mut offset = 0;
    let mut id = 0;

    for mut line in rdr.lines().filter_map(|l| l.ok()) {
        line.push('\n');

        if inflight == qd {
            eprintln!("Pausing for {} inflight writes...", inflight);
            while inflight > 0 {
                let cqe = iou.wait_for_cqe().unwrap();
                map.remove(&cqe.user_data());
                inflight -= 1;
            }
        }

        let io = IouMarker::new(vec![line.into_bytes()]);

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

    eprint!("Waiting for {} writes...", inflight);
    let _ = iou.wait_for_cqes(inflight.try_into().unwrap());
    eprintln!("done!");
}

fn main() {
    let v: Vec<u8> = vec![1, 2, 3];
    let iom = IoMarker::new(v);

    println!("{:p}, {:p}", &iom._data, &iom.iovec[0]);

    let ioum = IouMarker::new(vec![vec![1, 2, 3]]);

    //let args = std::env::args().collect::<Vec<_>>();
    //let f1 = &args[1];

    //let qd = if args.len() > 3 {
    //    args[2].parse().unwrap()
    //} else {
    //    128
    //};

    //copy(f1, qd);
}

//fn main() {
//    let infile = std::env::args().nth(1).expect("Pass an input file");
//    let qd = std::env::args()
//        .nth(2)
//        .unwrap_or("127".to_string())
//        .parse()
//        .unwrap();
//
//    let rdr = BufReader::new(File::open(infile).expect("Can't open file"));
//
//    let file = File::create("rust_testfile").unwrap();
//    let fd = file.as_raw_fd();
//    let mut inflight = 0;
//    let mut id = 0;
//    let mut offset = 0;
//
//    println!("Queue depth: {}", qd);
//
//    let mut iou = IoUring::new(qd).expect("Can't create ring");
//
//    let mut map = HashMap::new();
//
//    for mut line in rdr.lines().filter_map(|l| l.ok()) {
//        if inflight == qd {
//            eprintln!("Pausing for {} inflight writes...", inflight);
//            while inflight > 0 {
//                let _ = iou.wait_for_cqe();
//                inflight -= 1;
//            }
//        }
//
//        line.push('\n');
//
//        let io = IouMarker::new(vec![line.into_bytes()]);
//
//        let mut sqe = iou.next_sqe().unwrap();
//        sqe.set_user_data(id);
//        unsafe {
//            sqe.prep_write_vectored(fd, &io.iovecs, offset);
//            sqe.set_flags(SubmissionFlags::IO_LINK);
//            inflight += 1;
//            offset += io.byte_len();
//        }
//
//        map.insert(id, io);
//        id += 1;
//    }
//
//    eprint!("Waiting for {} writes...", inflight);
//    let _ = iou.wait_for_cqes(inflight.try_into().unwrap());
//    eprintln!("done!");
//}
