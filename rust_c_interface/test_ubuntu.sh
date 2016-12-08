cargo clean
cargo build --release 
cp ../target/release/libimmix_rust.so .
export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:../target/release/
gcc test.c libimmix_rust.so -lpthread
./a.out
