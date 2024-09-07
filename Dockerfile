# Use the official Rust image from DockerHub
FROM rust:latest

#install solana tools
RUN sh -c "$(curl -sSfL https://release.solana.com/v1.18.18/install)"
ENV PATH="/root/.local/share/solana/install/active_release/bin:$PATH"
#install protoc
RUN apt-get update && apt-get install -y protobuf-compiler

# Set the working directory inside the container
WORKDIR /app
