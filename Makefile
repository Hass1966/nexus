.PHONY: build run test lint fmt docker-up docker-down health-check migrate clean dev

build:
	cargo build --release

run:
	cargo run --bin nexus

dev:
	RUST_LOG=nexus=debug,tower_http=debug cargo run --bin nexus

test:
	cargo test --workspace

lint:
	cargo clippy --workspace -- -D warnings

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all -- --check

docker-up:
	docker compose up -d
	@echo "Waiting for services to be healthy..."
	@sleep 5
	@docker compose ps

docker-down:
	docker compose down

docker-clean:
	docker compose down -v

health-check:
	@bash scripts/health-check.sh

migrate:
	sqlx migrate run --source migrations

migrate-create:
	@read -p "Migration name: " name; \
	sqlx migrate add -r $$name --source migrations

clean:
	cargo clean

check: fmt-check lint test
	@echo "All checks passed."
