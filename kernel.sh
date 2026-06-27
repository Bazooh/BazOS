cargo fmt
cargo +nightly xtask
cd kernel || exit
cargo +nightly "$@"