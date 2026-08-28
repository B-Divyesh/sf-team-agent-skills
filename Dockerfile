FROM node:22-alpine AS frontend
WORKDIR /app
COPY package.json package-lock.json* ./
RUN npm install
COPY frontend ./frontend
COPY tsconfig.json vite.config.ts ./
RUN npm run build

FROM rust:1.85-slim AS backend
WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
COPY backend ./backend
COPY tests ./tests
RUN cargo build --release

FROM debian:bookworm-slim
ARG BUILD_SHA=dev
ENV BUILD_SHA=${BUILD_SHA} PORT=8080 DATABASE_PATH=/data/registry.db
RUN useradd --system --uid 10001 --create-home app && mkdir -p /data /app/dist && chown -R app:app /data /app
COPY --from=frontend /app/dist /app/dist
COPY --from=backend /app/target/release/team-agent-skills /app/team-agent-skills
USER app
WORKDIR /app
EXPOSE 8080
ENTRYPOINT ["/app/team-agent-skills"]
