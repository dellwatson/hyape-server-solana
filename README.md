# Game Server On-Chain with Ephemeral Rollups

This README is an auto generated progress update that explains how to build a simpler game server on-chain using Solana's Ephemeral Rollups.

## Building The Program

```bash
# Build the program
anchor build
```

## Generating Program ID

After building your program, you need to generate a program ID:

```bash
# Generate a new keypair for your program
solana-keygen new -o target/deploy/my_project-keypair.json

# Get the program ID
solana address -k target/deploy/my_project-keypair.json
```

## Updating Program ID

1. Copy the generated program ID
2. Update `Anchor.toml` with your program ID:
   ```toml
   [programs.localnet]
   my_project = "YOUR_PROGRAM_ID"
   ```
3. Update `declare_id!()` in your program's `lib.rs` file:
   ```rust
   declare_id!("YOUR_PROGRAM_ID");
   ```

## Rebuilding After ID Update

```bash
# Rebuild the program with the new ID
anchor build
```

## Deployment

### Testnet/Devnet Deployment

```bash
# Set Solana to devnet
solana config set --url devnet

# Airdrop some SOL to your wallet for transaction fees
solana airdrop 2

# Deploy to devnet
anchor deploy
```

## Common Issues and Solutions

- **Program ID Mismatch**: Ensure the program ID in `Anchor.toml` and `lib.rs` match exactly
- **Insufficient SOL**: Make sure your wallet has enough SOL for deployment
- **Build Errors**: Run `anchor clean` before rebuilding if you encounter persistent errors

## What are Ephemeral Rollups?

Ephemeral Rollups in Solana provide a way to offload computation from the base layer while maintaining security. They allow for:

- Faster transaction processing
- Lower costs
- Better user experience for interactive applications like games
- Maintaining security guarantees from the base layer

## Understanding Account Delegation

Delegation is the key mechanism that transfers control of an account from Solana's base layer to the Ephemeral Rollup.

### How Delegation Works

When an account is delegated:

1. The account's data is copied to the rollup
2. The account becomes "locked" on the base layer (can't be modified directly)
3. All operations on that account happen in the rollup until it's undelegated
4. The account's ownership on the base layer changes to the Delegation Program

### Important Notes About Delegation

- Delegations work at the account level, not at individual fields
- Only data accounts get delegated, not program accounts (executable code)
- After delegation, there are effectively two copies of the account:
  - The original (now locked) copy on the base layer
  - The working copy in the rollup that can be modified

## Implementation with Anchor

### The Delegate Attribute

Use the `#[delegate]` attribute macro to enable account delegation:

```rust
#[delegate]
#[derive(Accounts)]
pub struct DelegateInput<'info> {
    pub payer: Signer<'info>,
    /// CHECK: The account to delegate
    #[account(mut, del)]
    pub pda: AccountInfo<'info>,
}
```

Key points:

- The `#[delegate]` attribute generates code for account delegation
- The account being delegated must use `AccountInfo<'info>` (not a deserialized account type)
- Add `#[account(mut, del)]` to mark which account is being delegated

### The Ephemeral Attribute

Add the `#[ephemeral]` attribute to your program module to enable rollup functionality:

```rust
#[ephemeral]
#[program]
pub mod my_game {
    // Program functions here
}
```

## Common Issues and Solutions

### Using AccountInfo for Delegation

When delegating accounts, you must use `AccountInfo<'info>` rather than deserialized Anchor account types:

❌ **Incorrect**:

```rust
#[account(mut, del)]
pub game_state: Account<'info, GameState>,
```

✅ **Correct**:

```rust
#[account(mut, del)]
pub game_state: AccountInfo<'info>,
```

### Account Access After Delegation

Users interact with the program normally. The Anchor framework and Ephemeral Rollups SDK handle the routing:

- If an account is delegated, operations go to the rollup
- If not, they go to the base layer

## Design Considerations for Games

When designing your game:

1. Consider which accounts need delegation (typically game state accounts)
2. Separate independent state into different accounts if they need independent control
3. Structure your game state appropriately for the rollup environment
4. Design clear flows for delegation and undelegation of game accounts

## Comparison to Ethereum's Layer 2 Solutions

Ephemeral Rollups share similarities with Ethereum L2 solutions but with key differences:

**Similarities**:

- Both create environments for faster/cheaper transaction execution
- Both start with a copy of state from the original chain

**Differences**:

- Ephemeral Rollups operate at the account level, not the entire state
- The delegation mechanism is more granular in Solana
- The architecture leverages Solana's account model rather than Ethereum's global state model

## Next Steps

1. Start with a simple implementation (e.g., position data only)
2. Test thoroughly on the base layer before adding rollup functionality
3. Add delegation for your game state accounts
4. Implement game logic that works with delegated accounts
