A simple project to cause a SIGSEGV in iou-rs

Simply run like so:

```bash
# Cause a SIGSEGV
cargo run -- 3 2 1

# Block forever failing to read the final write
cargo run -- 3 2 0
```
