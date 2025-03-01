.PHONY: up
up:
	docker compose up -d

.PHONY: down
down:
	docker compose down

.PHONY: dev
dev: migrate
	cargo run --manifest-path my-todo/Cargo.toml

.PHONY: migrate
migrate:
	sqlx migrate run

.PHONY: test
test:
	cargo nextest run --manifest-path my-todo/Cargo.toml
