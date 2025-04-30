# solana_vault_cpi

This crate provides a CPI (Cross-Program Invocation) interface for the Solana Vault program, using Anchor framework bindings. It is designed for use in the Orderly Network and related Solana DeFi integrations.

## Features

- Anchor CPI bindings for Solana Vault
- Compatible with Solana 1.16.x and Anchor 0.28.x
- Auto-generated CPI interface from `idl.json`

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
solana_vault_cpi = "0.1.0" # or latest version
```

Import in your code:

```rust
use solana_vault_cpi::*;
```

## License

MIT
