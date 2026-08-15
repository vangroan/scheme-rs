@echo off

set "MIRIFLAGS=-Zmiri-backtrace=full"
cargo +nightly miri test
