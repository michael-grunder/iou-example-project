commit 1122c2e1e5b4484039e673b2e5fc01b42f579046
Author: michael-grunder <michael.grunder@gmail.com>
Date:   Thu Nov 21 14:21:08 2019 -0800

    Update readme

diff --git README.md README.md
index 28e2039..b4b4003 100644
--- README.md
+++ README.md
@@ -3,9 +3,10 @@ A simple project to cause a SIGSEGV in iou-rs
 Simply run like so:

 ```bash
-# Cause a SIGSEGV
-cargo run -- 3 2 1
+# Should replicate SIGSEGV
+git checkout master && cargo run -- 3 2 1
+
+# Using modified upstream
+git checkout sigsegv.peek-for-cqe && cargo run -- 3 2 1

-# Block forever failing to read the final write
-cargo run -- 3 2 0
 ```
