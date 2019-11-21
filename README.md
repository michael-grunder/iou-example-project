Segmentation fault demonstration

```bash
# Should replicate SIGSEGV
git checkout master && cargo run -- 3 2 1

# Using modified upstream
git checkout sigsegv.peek-for-cqe && cargo run -- 3 2 1
```
