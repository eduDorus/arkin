# DEFAULT ARGUMENTS
ARG BINARY_NAME=main
ARG PROFILE=maxperf

# BUILD IMAGE
FROM rust:1.86.0-bookworm AS build
# Set arguments
ARG BINARY_NAME
ARG PROFILE

# Set working directory
WORKDIR /app

# Optimise build time by caching dependencies
COPY . ./
# RUN mkdir -p arkin/src/bin && echo "fn main() {}" > src/bin/${BINARY_NAME}.rs
# RUN cargo build --profile ${PROFILE} --bin ${BINARY_NAME}

# Copy the source
# COPY src ./src
RUN cargo build --profile ${PROFILE} --bin ${BINARY_NAME}

# PRODUCTION IMAGE
FROM gcr.io/distroless/cc-debian12
# Set arguments
ARG BINARY_NAME
ARG PROFILE

# Set environment variables
ENV RUST_LOG=info

# Copy the binary from the build image
WORKDIR /app
COPY --from=build /app/target/${PROFILE}/${BINARY_NAME} /app