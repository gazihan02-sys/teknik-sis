.PHONY: setup backend frontend css dev clean

# Setup everything
setup:
	cd frontend && npm install
	rustup target add wasm32-unknown-unknown
	cargo install trunk

# Build TailwindCSS
css:
	cd frontend && npm run css:build

# Watch TailwindCSS
css-watch:
	cd frontend && npm run css:watch

# Run backend
backend:
	cp .env.example .env 2>/dev/null || true
	cargo run -p backend

# Run frontend with trunk
frontend: css
	cd frontend && trunk serve --open

# Build all
build:
	cargo build --workspace --exclude frontend
	cd frontend && trunk build --release

# Clean
clean:
	cargo clean
	rm -rf frontend/dist frontend/node_modules
