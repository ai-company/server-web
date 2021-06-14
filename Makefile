ALL:
	cargo run --release

watch:
	cargo watch -x "run"

.PHONY: watch
