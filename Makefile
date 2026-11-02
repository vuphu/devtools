.PHONY: all build build-ext run hot-reload clean

all: build build-ext

build:
	cargo build --release -p devtools

build-ext:
	cargo build --target wasm32-wasip3 --release -p timestamp
	mkdir -p "$${DEVTOOLS_HOME:-$$HOME/.devtools}/extensions"
	cp target/wasm32-wasip3/release/timestamp.wasm "$${DEVTOOLS_HOME:-$$HOME/.devtools}/extensions/timestamp.wasm"

run: build build-ext
	./target/release/devtools

hot-reload:
	@test -x .tools/bin/cargo-watch || cargo install cargo-watch --locked --root .tools
	@.tools/bin/cargo-watch watch -w apps -w packages -w extensions -s 'cargo run -p devtools'

clean:
	cargo clean
