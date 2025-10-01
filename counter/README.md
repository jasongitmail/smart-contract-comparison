# Counter Smart Contracts

Simple on-chain counter implementations for Ethereum (Solidity) and Solana (Rust), demonstrating integer storage, state modification, and access control.

## Overview

Both contracts implement a counter that:
- Stores an integer value on-chain
- Allows increment/decrement operations
- Restricts modifications to the contract owner/deployer only
- Prevents underflow (cannot go below zero)
- Protects against overflow

## Files

- **Counter.sol** - Ethereum smart contract
- **counter.rs** - Solana program

## Functionality

### Ethereum (Solidity) - `Counter.sol`

**State:**
- `count` (uint256) - Current counter value
- `owner` (address, immutable) - Deployer address set in constructor

**Functions:**
- `increment()` - Adds 1 to counter (owner-only)
- `decrement()` - Subtracts 1 from counter (owner-only, fails if count is 0)
- `getCount()` - Returns current value (view, no gas)
- `getOwner()` - Returns owner address (view, no gas)

**Events:**
- `Incremented(uint256 newCount)`
- `Decremented(uint256 newCount)`

### Solana (Rust) - `counter.rs`

**Account Data:**
- `is_initialized` (bool) - Initialization flag
- `count` (u64) - Current counter value
- `owner` (Pubkey) - Owner's public key

**Instructions:**
- `Initialize` - Set up counter account with owner
- `Increment` - Adds 1 to counter (owner-only)
- `Decrement` - Subtracts 1 from counter (owner-only, fails if count is 0)

**Required Accounts:**
- Account 0: Counter account (writable, program-owned)
- Account 1: Owner/signer account

## Security Features

### Access Control
- ✅ **Owner-only modifications**: Only deployer can increment/decrement
- ✅ **Immutable ownership** (Solidity): Owner cannot be changed
- ✅ **Signer verification**: All state changes require valid signature
- ✅ **Initialization protection** (Solana): Prevents double initialization

### Integer Safety
- ✅ **Overflow protection**: Built-in (Solidity ^0.8.0) and checked_add (Rust)
- ✅ **Underflow protection**: Explicit checks prevent negative values
- ✅ **Type safety**: uint256 (Solidity) and u64 (Rust)

### State Integrity
- ✅ **Initialization checks**: Validates account setup before operations
- ✅ **Program ownership verification** (Solana): Ensures program controls account
- ✅ **Writable flag checks** (Solana): Confirms account can be modified
- ✅ **Proper error handling**: Clear, specific error messages

## Gas/Compute Costs

### Read Operations (No Cost for External Calls)
- **Ethereum**: `getCount()` and `getOwner()` are `view` functions - no gas when called externally via RPC
- **Solana**: Read counter value off-chain via account data fetch - no transaction needed

### Write Operations (Incur Costs)
- **Ethereum**: `increment()` and `decrement()` modify state - ~26k-45k gas
- **Solana**: All instructions require transactions - ~5000 compute units

## Key Differences

| Aspect | Ethereum | Solana |
|--------|----------|--------|
| **Initialization** | Constructor auto-runs on deploy | Explicit Initialize instruction required |
| **Owner Storage** | `immutable` keyword prevents changes | Owner stored in account data (could be transferable) |
| **Integer Type** | uint256 (256-bit) | u64 (64-bit) |
| **Overflow Check** | Built-in Solidity ^0.8.0 | `checked_add()` / `checked_sub()` |
| **Read Pattern** | View functions (gas-free externally) | Off-chain RPC queries |
| **Access Control** | Modifier pattern (`onlyOwner`) | Manual verification in each function |
| **Events** | Emitted and indexed | Logs via `msg!()` macro |

## Usage Examples

### Solidity (Ethereum)
```javascript
// Deploy
const Counter = await ethers.deployContract("Counter");

// Increment (costs gas)
await counter.increment();

// Get count (no gas for external calls)
const count = await counter.getCount();
console.log(count); // 1

// Non-owner tries to increment (fails)
await counter.connect(otherUser).increment(); // Reverts
```

### Rust (Solana)
```bash
# Deploy program
solana program deploy counter.so

# Create counter account
solana-keygen new -o counter-account.json

# Initialize counter
# (build and send Initialize instruction)

# Increment counter
# (build and send Increment instruction with owner signature)

# Read count (no transaction)
solana account <counter-account-pubkey>
```

## Testing

### Solidity
- Tests should verify owner-only access
- Test overflow protection (try incrementing from max value)
- Test underflow protection (try decrementing from zero)

### Rust
- `test_initialize()` - Verifies initialization logic
- `test_increment_overflow_protection()` - Ensures checked_add prevents overflow
- Add tests for owner verification and underflow protection

## Production Considerations

Both contracts are production-ready with:
- ✅ Comprehensive security measures
- ✅ Gas/compute optimization
- ✅ Clear error messages
- ✅ Access control implementation

**Enhancements for production:**
- Consider ownership transfer functionality
- Add reset() function if needed
- Implement increment/decrement by custom amounts
- Add pause/unpause functionality for emergency stops
- Comprehensive integration testing
- Professional security audit

## License

MIT
