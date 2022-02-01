.PHONY: test run build check clean fix release

default: test

test: fix
	cargo test --all-features --lib

run:
	cargo run

build:
	cargo build

check:
	cargo check --all
	cargo deny check

clean:
	cargo clean

fix:
	cargo fix --allow-dirty --allow-staged
	cargo fmt -- --check
	cargo clippy --all-targets --all-features --tests --benches -- -D warnings

doc:
	cargo doc --all-features --no-deps

release: test doc
	cargo build --release

simulate:
	RUST_LOG=tx_coordinator=debug cargo run -- --cfg ./tests/cfg.toml --dir ./tests/sqlfiles


cmd =
ifeq "$(docker-compose)" ""
  cmd = lima nerdctl compose
endif

db:
	cd tests/docker && $(cmd) -f ./docker-compose.yml up

# need run make db first
it-test:
	cargo test --test integration -- --test-threads=1
