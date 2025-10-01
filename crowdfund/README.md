# Crowdfund Smart Contracts

## Concept

A time-based crowdfunding system where:

1. **Campaign Creation**: Owner sets a funding goal and deadline (block height in the future)
2. **Open Contributions**: Anyone can contribute funds during the active campaign period
3. **Success Path**: If the goal is reached by the deadline, the owner can withdraw all funds
4. **Failure Path**: If the goal is NOT reached by the deadline, contributors can claim full refunds
5. **Transaction Fees**: On Ethereum, refunds are full amounts (gas paid separately). On Solana, contributors pay transaction fees for their refund claim

This is a classic "all-or-nothing" crowdfunding model similar to Kickstarter, implemented on both Ethereum and Solana.

## Files

- **Crowdfund.sol** - Ethereum smart contract
- **crowdfund.rs** - Solana program

## Functionality

### Ethereum (Solidity) - `Crowdfund.sol`

**State:**
- `owner` (address, immutable) - Campaign creator
- `goal` (uint256, immutable) - Funding target in wei
- `deadline` (uint256, immutable) - Block number when campaign ends
- `totalRaised` (uint256) - Current total contributions
- `finalized` (bool) - Whether campaign has been finalized
- `contributions` (mapping) - Tracks each contributor's amount

**Functions:**
- `contribute()` - Send ETH to contribute (payable)
- `isSuccessful()` - Check if goal was reached (view)
- `withdraw()` - Owner withdraws funds if successful
- `refund()` - Contributor claims refund if failed
- `finalize()` - Mark failed campaign as finalized (optional)

**Events:**
- `Contributed(address indexed contributor, uint256 amount)`
- `GoalReached(uint256 totalRaised)`
- `Refunded(address indexed contributor, uint256 amount)`
- `Withdrawn(address indexed owner, uint256 amount)`

### Solana (Rust) - `crowdfund.rs`

**Campaign Account:**
- `is_initialized` (bool)
- `owner` (Pubkey)
- `goal` (u64) - Target in lamports
- `deadline` (u64) - Slot number
- `total_raised` (u64)
- `finalized` (bool)

**Contributor Account:**
- `amount` (u64) - Contribution amount

**Instructions:**
- `Initialize { goal, duration_slots }` - Create campaign
- `Contribute { amount }` - Add contribution
- `Withdraw` - Owner claims funds if successful
- `Refund` - Contributor claims refund if failed

**Required Accounts:**
- Campaign account (stores campaign state)
- Contributor record accounts (one per contributor)
- Owner/contributor signers
- System program (for transfers)

## Security Features

### Reentrancy Protection
- ✅ **Checks-Effects-Interactions** (Solidity): State updated before external calls
- ✅ **Pull-based refunds**: Contributors call refund(), no pushing to addresses
- ✅ **Finalization flag**: Prevents double-withdrawal or double-refund

### Time-Based Access Control
- ✅ **Deadline enforcement**: Contributions only during active period
- ✅ **Post-deadline actions**: Withdraw/refund only after deadline
- ✅ **Immutable deadlines**: Cannot be changed after deployment

### Financial Integrity
- ✅ **Overflow protection**: Built-in (Solidity) and checked_add (Rust)
- ✅ **Zero-contribution prevention**: Rejects empty contributions
- ✅ **Goal validation**: Must be greater than zero
- ✅ **Balance tracking**: Accurate contributor accounting

### State Management
- ✅ **Initialization checks**: Validates proper setup
- ✅ **Signer verification**: All state changes require signatures
- ✅ **Owner verification**: Only owner can withdraw on success
- ✅ **Success/failure separation**: Withdraw and refund are mutually exclusive

### Ethereum-Specific
- ✅ **Immutable parameters**: goal and deadline cannot change
- ✅ **Safe external calls**: Uses low-level call with success check
- ✅ **Event emission**: Full transparency of all actions

### Solana-Specific
- ✅ **Program ownership checks**: Validates account ownership
- ✅ **Writable validations**: Ensures accounts can be modified
- ✅ **Clock sysvar usage**: Reliable time source (slots)
- ✅ **Lamport transfer safety**: Direct lamport manipulation with proper checks

## Key Differences

| Aspect | Ethereum | Solana |
|--------|----------|--------|
| **Time Measurement** | Block number | Slot number |
| **Contribution** | `msg.value` (payable) | Explicit amount + transfer instruction |
| **Storage** | Single contract stores all | Campaign + per-contributor accounts |
| **Refunds** | Pull-based, full amount | Pull-based, contributor pays tx fee |
| **Funds Holding** | Contract balance | Campaign account lamports |
| **Goal/Deadline** | Immutable (constructor) | Set on Initialize instruction |
| **Success Check** | View function (free) | Read campaign account data off-chain |

## Usage Examples

### Solidity (Ethereum)

```javascript
// Deploy campaign: 10 ETH goal, 1000 blocks duration
const Crowdfund = await ethers.deployContract("Crowdfund", [
  ethers.parseEther("10"),
  1000
]);

// Contribute 1 ETH
await crowdfund.contribute({ value: ethers.parseEther("1") });

// Check status
const raised = await crowdfund.totalRaised();
const successful = await crowdfund.isSuccessful();

// After deadline - if successful
await crowdfund.withdraw(); // Owner only

// After deadline - if failed
await crowdfund.refund(); // Each contributor
```

### Rust (Solana)

```bash
# Deploy program
solana program deploy crowdfund.so

# Initialize campaign (10 SOL goal, 10000 slots duration)
# Creates campaign account, sets owner
# (client must build and send Initialize instruction)

# Contribute 1 SOL
# (client builds Contribute instruction with amount)
# Creates/updates contributor record account

# After deadline - if successful
# Owner calls Withdraw instruction
# Transfers lamports from campaign to owner

# After deadline - if failed
# Each contributor calls Refund instruction
# Transfers lamports from campaign to contributor
```

## Testing Considerations

### Critical Test Cases
1. **Contribution validation**:
   - Before deadline ✓
   - After deadline ✗
   - Zero amount ✗

2. **Withdrawal**:
   - Owner only ✓
   - After deadline ✓
   - Goal reached ✓
   - Already finalized ✗

3. **Refunds**:
   - After deadline ✓
   - Goal not reached ✓
   - Valid contribution exists ✓
   - Already refunded ✗

4. **Edge cases**:
   - Exact goal match
   - Multiple contributions from same address
   - Overflow protection
   - Reentrancy attempts

## Transaction Fee Considerations

### Ethereum
- **Contributors pay**: Gas for contribute() and refund() calls
- **Refund amount**: Full original contribution
- **Owner pays**: Gas for withdraw() call

### Solana
- **Contributors pay**: Transaction fee for Contribute and Refund instructions
- **Refund amount**: Full original contribution minus transaction fee for refund claim
- **Owner pays**: Transaction fee for Withdraw instruction
- **Account rent**: Campaign and contributor accounts must maintain rent-exemption

## Production Enhancements

For production deployment, consider:
- **Emergency pause**: Add ability to pause contributions
- **Partial withdrawals**: Allow owner to withdraw as goal is reached
- **Contribution limits**: Min/max per contributor
- **Campaign extension**: Allow deadline extension with rules
- **Multiple goals**: Stretch goals with different tiers
- **Token support**: Accept ERC20 tokens (Ethereum) or SPL tokens (Solana)
- **Refund automation**: Batch refund processing
- **Comprehensive testing**: Full integration test suite
- **Professional audit**: Security review before mainnet deployment

## License

MIT
