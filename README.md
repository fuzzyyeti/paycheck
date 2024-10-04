# Paycheck
Radar hackathon project in progress.

## Problem Statement 
I want to store my mined ORE or LSTs on a hardware wallet, but I want some money to pay for
my daily expenses. I don't want to use my hardware wallet every week to swap for USDC and
send to my off-ramp.

## Solution
Paycheck allows you to sign one Tx with your hardware wallet, and set up an interval and amount to 
swap and send.

## Project Structure

This project is divided into three main parts:

1. **Solana Program** (`solana`): Contains the Solana on-chain program.
2. **Bot** (`bot`): Contains the bot logic.
3. **Web** (`web`): Contains the web frontend.

Additionally, there are two support crates:

- **mpl-macros**: Contains macros copied from the Metaplex library with dependencies removed. 
- **whirlpool-state**: Contains state definitions from orca-so/whirlpools with the dependencies removed.

## Getting Started

### Prerequisites

- Rust and Cargo
- Solana CLI
- Copy the .env.example file to .env and fill in the required values

## Building and testing the solana program

```sh
cd paycheck
cargo test-sbf
```

## Running the web app
```sh
git clone https://github.com/regolith-labs/solana-playground ../solana-playground
```
Install dioxus CLI
```sh
cargo install dioxus
```

Run the web app
```sh
cd web
dx serve --hot-reload
```

## Running the bot
Make sure you have some SOL on your BOT_KEY.
```sh
cargo run -p bot
```