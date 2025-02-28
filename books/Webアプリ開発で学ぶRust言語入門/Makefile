.PHONY: up
up:
	cd my-todo && docker-compose up -d

.PHONY: down
down:
	cd my-todo && docker-compose down

.PHONY: dev
dev: migrate
	cd my-todo && cargo run

.PHONY: migrate
migrate:
	cd my-todo && sqlx migrate run

.PHONY: test
test:
	cd my-todo && cargo nextest run
