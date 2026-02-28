## Control center
- click dropdown :
  ```
  thread 'main' (3624292) panicked at /home/nia/.cargo/git/checkouts/zed-edf0a265c77bbee2/b310d0d/crates/gpui/src/window.rs:2475:9:
  assertion `left == right` failed: cannot call defer_draw during deferred drawing
    left: 1
  right: 0
  stack backtrace:
    0:     0x555559e96cda - std[f9370ac21d0f083b]::backtrace_rs::backtrace::libunwind::trace
                                at /rustc/f889772d6500faebcac5bb70fa44b5e6581c38cd/library/std/src/../../backtrace/src/backtrace/libunwind.rs:117:9
    1:     0x555559e96cda - std[f9370ac21d0f083b]::backtrace_rs::backtrace::trace_unsynchronized::<std[f9370ac21d0f083b]::sys::backtrace::_print_fmt::{closure#1}>
                                at /rustc/f889772d6500faebcac5bb70fa44b5e6581c38cd/library/std/src/../../backtrace/src/backtrace/mod.rs:66:14
    2:     0x555559e96cda - std[f9370ac21d0f083b]::sys::backtrace::_print_fmt
                                at /rustc/f889772d6500faebcac5bb70fa44b5e6581c38cd/library/std/src/sys/backtrace.rs:74:9
    3:     0x555559e96cda - <<std[f9370ac21d0f083b]::sys::backtrace::BacktraceLock>::print::DisplayBacktrace as core[7a9a0e10e2660de0]::fmt::Display>::fmt
  ```

## Chat
- apparition quand une app est fullscreen le chat devrais faire quasiement 100 pourcent en hauteur