lib := st/libst.so

clippy: $(lib)
	cargo +nightly clippy

st/libst.so: $(shell find st -name '*.[ch]')
	cd st && make libst.so

run: $(lib)
	cargo run

dummy:
	find src -name '*.rs' -exec grep -nHA1 '^// DUMMY' {} \;
