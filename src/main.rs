fn stall_nop(queue_depth: u32, write_count: usize) {
    let mut writes = 0;
    let mut reads = 0;
    let mut counter = 0;

    let mut iou = iou::IoUring::new(queue_depth).unwrap();

    println!("{:>3} {:>6} {:>6} {:>6}", "IDX", "OP", "READS", "WRITES");

    while writes < write_count {
        let op = match iou.next_sqe() {
            Some(mut sqe) => {
                unsafe {
                    sqe.prep_nop();
                }
                writes += 1;
                "WRITE"
            }
            None => {
                let _ = iou.wait_for_cqe().unwrap();
                reads += 1;
                "READ"
            }
        };

        counter += 1;
        println!("{:>3} {:>6} {:>6} {:>6}", counter, op, reads, writes);
    }

    while reads < write_count {
        let _ = iou.wait_for_cqe().unwrap();
        reads += 1;
        println!("{:>3} {:>6} {:>6} {:>6}", counter, "READ", reads, writes);
    }
}

fn main() {
    let queue_depth: u32 = std::env::args()
        .nth(1)
        .unwrap_or("2".to_string())
        .parse()
        .unwrap();

    let write_count: usize = std::env::args()
        .nth(2)
        .unwrap_or("3".to_string())
        .parse()
        .unwrap();

    println!("QUEUE DEPTH: {}, WRITE COUNT: {}", queue_depth, write_count);

    stall_nop(queue_depth, write_count);
}
