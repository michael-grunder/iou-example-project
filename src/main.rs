use iou::IoUring;
use std::{fs::File, io::IoSlice, os::unix::io::AsRawFd};

fn main() {
    let words = vec!["some\n".as_bytes().to_vec(), "data\n".as_bytes().to_vec()];

    let file = File::create("rust_testfile").unwrap();
    let fd = file.as_raw_fd();
    let mut index = 0;
    let mut offset = 0;

    let mut iou = IoUring::new(32).expect("Can't create ring");

    for word in words.into_iter() {
        // How to keep around *both* the `word` vec and the boxed slice container?
        let buf = Box::new([IoSlice::new(&word)]);
        let len = word.len();

        let mut sqe = iou.next_sqe().unwrap();
        unsafe {
            sqe.prep_write_vectored(fd, &*buf, offset);
            sqe.set_user_data(index);
        }

        std::mem::forget(buf);
        std::mem::forget(word);

        offset += len;
        index += 1;
    }

    while index > 0 {
        let cqe = iou.wait_for_cqe().unwrap();
        println!("Now I could free write id: {}", cqe.user_data());
        index -= 1;
    }
}
