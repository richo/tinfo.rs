target/tinfo: src/main.rs
	cargo build

install: target/debug/tinfo
	cp $< /usr/local/bin
