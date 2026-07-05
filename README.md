# worldline_api

Serde models for the [Worldline NAM Payment API](https://docs.na.worldline-solutions.com/) — payment profiles, cards, bank accounts, and batch settlement.

This crate is **standalone**: it has no Mae, Statbook, or other in-house dependencies. It is intended for open-source use by any Rust developer integrating with Worldline.

## Installation

```toml
[dependencies]
worldline_api = "0.1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

> **crates.io**: not yet published. Until then, use a git dependency:
>
> ```toml
> worldline_api = { git = "https://github.com/Mae-Technologies/worldline_api.git", branch = "main" }
> ```

## Usage

The crate provides wire-format structs and helpers — it does **not** perform HTTP calls. Your service builds JSON from these types (or deserializes Worldline responses into them).

```rust
use worldline_api::profile::{format_card_label, redact_card_number, BankAccount, BankAccountType};

let display = redact_card_number("4030001234567890");
let label = format_card_label(Some("VI"), "4030001234567890", "Jane Doe");
```

Validate billing and bank payloads before sending to Worldline:

```rust
use worldline_api::profile::{BankAccount, BillingAddress};

// BillingAddress and BankAccount deserialize from JSON; fields match Worldline docs.
billing.validate_for_worldline()?;
bank.validate_for_worldline()?;
```

## Modules

| Module | Purpose |
|--------|---------|
| `profile` | Create/update profile bodies, card tokens, billing, EFT bank accounts, GET profile details |
| `batch` | Batch criteria and success/error responses |

## Development

See [DEVELOPMENT.md](DEVELOPMENT.md) for `rustfmt`, `clippy`, `cargo-deny`, coverage thresholds, and local smoke tests.

```bash
cargo test
bash scripts/smoke-test.sh
```

## License

MIT — see [LICENSE](LICENSE).