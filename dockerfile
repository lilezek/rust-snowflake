FROM rust
   
# Copy the binary from rust
COPY . /app

# Set the working directory
WORKDIR /app

RUN cargo build --release

# Expose the port
EXPOSE 3000

# Set the entrypoint
ENTRYPOINT ["/app/target/release/rust-snowflake"]