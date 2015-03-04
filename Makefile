target/tinfo: src/main.rs
	cargo build

install: target/tinfo
	cp $< /usr/local/bin
