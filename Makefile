.PHONY: dev build release ingest test test-engine test-integridade verificar fmt clean

dev:
	trunk serve

release: build

build:
	trunk build --release

ingest:
	python3 tools/ingest.py

test: test-engine test-integridade

test-engine:
	cargo test --lib

test-integridade:
	cargo test --test questoes_integridade -- --nocapture

verificar:
	./scripts/verificar.sh

fmt:
	cargo fmt

clean:
	rm -rf dist target