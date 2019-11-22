use std::{fs::File, io::IoSlice, os::unix::io::AsRawFd};

unsafe fn write_chunk(iou: &mut iou::IoUring, fd: i32, data: &[IoSlice], offset: &mut usize) {
    let mut sqe = iou.next_sqe().unwrap();
    sqe.prep_write_vectored(fd, &data, *offset);
    *offset += data.len();
}

fn read_chunk(iou: &mut iou::IoUring) -> usize {
    let cqe = iou.wait_for_cqe().unwrap();
    cqe.result().unwrap()
}

fn main() {
    let mut iou = iou::IoUring::new(2).unwrap();

    let file = File::create("rust_testfile").unwrap();
    let fd = file.as_raw_fd();

    let mut offset = 0;
    let slice = [IoSlice::new(b"012345678\n")];
    
    for _ in 0..2 {
        unsafe { write_chunk(&mut iou, fd, &slice, &mut offset); }
    }

    read_chunk(&mut iou);
    unsafe { write_chunk(&mut iou, fd, &slice, &mut offset); }

    while let Some(cqe) = iou.peek_for_cqe() {
        println!("beep boop: {:?}", cqe.user_data());
    }
}
