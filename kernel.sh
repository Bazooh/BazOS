cargo fmt
cargo +nightly xtask || exit
cd kernel || exit
cargo +nightly "$@"