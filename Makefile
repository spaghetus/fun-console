all: $(shell find examples src -name *.asm | sed 's/\.asm/\.bin/g') target/release/fun

target/release/fun: src/main.rs src/bios.bin
	cargo build --release

%.bin: %.asm
	64tass -q -B -b --tab-size=1 -o $@ $<