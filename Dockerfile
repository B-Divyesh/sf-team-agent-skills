FROM node:22-alpine AS frontend
WORKDIR /app
COPY package.json package-lock.json* ./
RUN npm ci
COPY frontend ./frontend
COPY tsconfig.json vite.config.ts ./
RUN npm run build

FROM rust:1-slim AS backend
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY backend ./backend
COPY tests ./tests
RUN cargo build --release --locked

FROM debian:bookworm-slim
ARG BUILD_SHA=dev
ARG GIT_SHA=dev
ARG SOURCE_COMMIT=dev
ENV BUILD_SHA=${BUILD_SHA} PORT=8080 DATABASE_PATH=/data/registry.db
LABEL org.opencontainers.image.revision=${SOURCE_COMMIT}
RUN useradd --system --uid 10001 --create-home app && mkdir -p /data /app/dist && chown -R app:app /data /app
COPY --from=frontend /app/dist /app/dist
COPY --from=backend /app/target/release/team-agent-skills /app/team-agent-skills
USER app
WORKDIR /app
EXPOSE 8080
ENTRYPOINT ["/app/team-agent-skills"]
