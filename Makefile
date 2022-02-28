test:
	cargo test --all
	cargo test -p vmm

check:
	cargo fmt --all -- --check