tinfo: src/tinfo.rs
	rustc -o $@ $^

install: tinfo
	cp $< /usr/local/bin
