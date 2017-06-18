all:
	cargo rustc --release -- -A overflowing-literals
clean:
	rm -rf target
