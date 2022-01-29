.PHONY: test run build check clean fix release

default: test

test: fix
	cargo test --all-features

run:
	cargo run

build:
	cargo build

check:
	cargo check --all
	cargo deny check

clean:
	cargo clean

fix: doc
	cargo fix --allow-dirty --allow-staged
	cargo fmt -- --check
	cargo clippy --all-targets --all-features --tests --benches -- -D warnings

doc:
	cargo doc --all-features --no-deps

release:
	cargo build --release

simulate:
	RUST_LOG=info cargo run -- --cfg ./tests/cfg.toml --dir ./tests/sqlfiles