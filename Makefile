st/libst.so:
	cd st && make libst.so

run: st/libst.so
	cargo run
