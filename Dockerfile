# ANF-008: Docker-Image für Railway.
#
# Stufen:
#   css     – Tailwind + daisyUI: styles/output.css erzeugen (Node.js)
#   chef    – Rust mit cargo-chef und cargo-leptos (Grundlage der Rust-Stufen)
#   planer  – Rezept (recipe.json) aus Cargo.toml und Cargo.lock schreiben
#   bau     – zuerst nur die Bibliotheken übersetzen (zwischengespeichert), dann das ERP
#   laufzeit – schlankes Image nur mit Serverprogramm und site/ (ohne cargo, node, Quellcode)
#
# Lokal ausprobieren:
#   docker build -t erp-saintwhoo .
#   docker run --rm -p 8080:8080 -e PORT=8080 -e DATABASE_URL=postgres://… erp-saintwhoo

ARG RUST_VERSION=1.97
ARG NODE_VERSION=22

# ---------------------------------------------------------------- css
FROM node:${NODE_VERSION}-bookworm-slim AS css
WORKDIR /app
COPY package.json package-lock.json ./
RUN npm ci
# Tailwind sucht die Klassen in src/ (siehe @source in styles/app.css)
COPY styles ./styles
COPY src ./src
RUN npm run css:build

# ---------------------------------------------------------------- chef
FROM rust:${RUST_VERSION}-bookworm AS chef
RUN rustup target add wasm32-unknown-unknown \
    && cargo install --locked cargo-chef@0.1.78 cargo-leptos@0.3.12
WORKDIR /app

# ---------------------------------------------------------------- planer
FROM chef AS planer
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# ---------------------------------------------------------------- bau
FROM chef AS bau
# Leptos: Komponenten-Typen löschen. .cargo/config.toml setzt das nur für Apple-Rechner; unter Linux
# bricht der Release-Bau des Servers sonst mit „queries overflow the depth limit“ ab.
# Muss vor `cargo chef cook` stehen, damit Bibliotheken und ERP mit denselben Optionen übersetzt werden.
ENV RUSTFLAGS="--cfg erase_components"
COPY --from=planer /app/recipe.json recipe.json
# Bibliotheken mit denselben Einstellungen übersetzen, die cargo-leptos verwendet.
# Dieser Schritt hängt nur von recipe.json ab und wird bei Änderungen in src/ wiederverwendet (AK-17).
# Server (ssr):
RUN cargo chef cook --release --no-default-features --features ssr \
    --recipe-path recipe.json
# Browser (hydrate, WebAssembly):
RUN cargo chef cook --profile wasm-release --target wasm32-unknown-unknown \
    --target-dir target/front --no-default-features --features hydrate \
    --recipe-path recipe.json

COPY . .
COPY --from=css /app/styles/output.css styles/output.css
RUN cargo leptos build --release

# ---------------------------------------------------------------- laufzeit
FROM debian:bookworm-slim AS laufzeit
RUN useradd --system --create-home --uid 10001 erp
WORKDIR /app

# Stammzertifikate für TLS-Verbindungen (z. B. öffentliche Datenbank-Adresse) aus der Bau-Stufe,
# damit die Laufzeit-Stufe kein apt-get braucht
COPY --from=bau /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt

COPY --from=bau /app/target/release/erp-saintwhoo /app/erp-saintwhoo
COPY --from=bau /app/target/site /app/site
# Leptos liest die Einstellungen aus [package.metadata.leptos]
COPY --from=bau /app/Cargo.toml /app/Cargo.toml

ENV LEPTOS_ENV=PROD \
    LEPTOS_SITE_ROOT=site \
    LEPTOS_SITE_ADDR=0.0.0.0:8080 \
    RUST_LOG=info
# Railway setzt PORT; ohne PORT hört der Server auf 8080.
EXPOSE 8080

USER erp
CMD ["/app/erp-saintwhoo"]
