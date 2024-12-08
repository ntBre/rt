st/libst.so: $(shell find st -name '*.[ch]')
	cd st && make libst.so

run: st/libst.so
	cargo run
