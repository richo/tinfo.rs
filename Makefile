target/debug/tinfo: src/main.rs
	cargo build

target/release/tinfo: src/main.rs
	cargo build --release

install: target/release/tinfo
	cp $< /usr/local/bin
