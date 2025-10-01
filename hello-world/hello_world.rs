use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};
use borsh::{BorshDeserialize, BorshSerialize};

/// Maximum message length (280 characters, similar to Twitter)
pub const MAX_MESSAGE_LENGTH: usize = 280;

/// Define the type of state stored in accounts
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct HelloWorldAccount {
    /// Flag to track if the account has been initialized
    pub is_initialized: bool,
    /// The stored message
    pub message: String,
    /// The public key of the last updater
    pub last_updater: Pubkey,
}

/// Define the program entrypoint
entrypoint!(process_instruction);

/// Program entrypoint's implementation
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    msg!("Hello World Solana program entrypoint");

    // Deserialize the instruction data to get the new message
    let instruction = HelloWorldInstruction::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    match instruction {
        HelloWorldInstruction::SetMessage { message } => {
            set_message(program_id, accounts, message)
        }
        HelloWorldInstruction::GetMessage => {
            get_message(accounts)
        }
    }
}

/// Instruction enum for the program
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum HelloWorldInstruction {
    /// Set a new message
    /// Accounts expected:
    /// 0. `[writable]` The account to store the message
    /// 1. `[signer]` The account of the person setting the message
    SetMessage { message: String },

    /// Get the current message (read-only)
    /// Accounts expected:
    /// 0. `[readable]` The account storing the message
    GetMessage,
}

/// Set a new message in the account
fn set_message(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    new_message: String,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let account = next_account_info(accounts_iter)?;
    let updater = next_account_info(accounts_iter)?;

    // Verify that the account is owned by this program
    if account.owner != program_id {
        msg!("Account does not have the correct program id");
        return Err(ProgramError::IncorrectProgramId);
    }

    // Verify that the account is writable
    if !account.is_writable {
        msg!("Account must be writable");
        return Err(ProgramError::InvalidAccountData);
    }

    // Verify that the updater is a signer
    if !updater.is_signer {
        msg!("Updater must be a signer");
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Validate message length
    if new_message.is_empty() {
        msg!("Message cannot be empty");
        return Err(ProgramError::InvalidInstructionData);
    }
    if new_message.len() > MAX_MESSAGE_LENGTH {
        msg!("Message too long (max {} bytes)", MAX_MESSAGE_LENGTH);
        return Err(ProgramError::InvalidInstructionData);
    }

    // Create or update the account data
    let hello_world_account = HelloWorldAccount {
        is_initialized: true,
        message: new_message.clone(),
        last_updater: *updater.key,
    };

    // Calculate required size
    let required_size = hello_world_account.try_to_vec()?.len();
    if account.data_len() < required_size {
        msg!("Account data size insufficient: {} < {}", account.data_len(), required_size);
        return Err(ProgramError::AccountDataTooSmall);
    }

    // Serialize and save the data
    hello_world_account.serialize(&mut &mut account.data.borrow_mut()[..])?;

    msg!("Message updated to: {}", new_message);
    msg!("Updated by: {}", updater.key);

    Ok(())
}

/// Get the current message from the account
/// Note: In production, reading data should be done off-chain via RPC calls
/// This instruction is included for demonstration purposes only
fn get_message(accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let account = next_account_info(accounts_iter)?;

    // Deserialize the account data
    let hello_world_account = HelloWorldAccount::try_from_slice(&account.data.borrow())?;

    // Check if account is initialized
    if !hello_world_account.is_initialized {
        msg!("Account has not been initialized");
        return Err(ProgramError::UninitializedAccount);
    }

    msg!("Current message: {}", hello_world_account.message);
    msg!("Last updated by: {}", hello_world_account.last_updater);

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use solana_program::clock::Epoch;
    use std::mem;

    #[test]
    fn test_hello_world() {
        let program_id = Pubkey::default();
        let key = Pubkey::default();
        let updater_key = Pubkey::default();
        let mut lamports = 0;
        let mut data = vec![0; 1000]; // Allocate sufficient space

        let account = AccountInfo::new(
            &key,
            false,
            true,
            &mut lamports,
            &mut data,
            &program_id, // Account should be owned by the program
            false,
            Epoch::default(),
        );

        let mut updater_lamports = 0;
        let mut updater_data = vec![];
        let updater_account = AccountInfo::new(
            &updater_key,
            true,
            false,
            &mut updater_lamports,
            &mut updater_data,
            &program_id,
            false,
            Epoch::default(),
        );

        let accounts = vec![account, updater_account];

        let instruction = HelloWorldInstruction::SetMessage {
            message: "Hello, Solana!".to_string(),
        };
        let instruction_data = instruction.try_to_vec().unwrap();

        let result = process_instruction(&program_id, &accounts, &instruction_data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_message_too_long() {
        let program_id = Pubkey::default();
        let key = Pubkey::default();
        let updater_key = Pubkey::default();
        let mut lamports = 0;
        let mut data = vec![0; 1000];

        let account = AccountInfo::new(
            &key,
            false,
            true,
            &mut lamports,
            &mut data,
            &program_id,
            false,
            Epoch::default(),
        );

        let mut updater_lamports = 0;
        let mut updater_data = vec![];
        let updater_account = AccountInfo::new(
            &updater_key,
            true,
            false,
            &mut updater_lamports,
            &mut updater_data,
            &program_id,
            false,
            Epoch::default(),
        );

        let accounts = vec![account, updater_account];

        let instruction = HelloWorldInstruction::SetMessage {
            message: "a".repeat(MAX_MESSAGE_LENGTH + 1),
        };
        let instruction_data = instruction.try_to_vec().unwrap();

        let result = process_instruction(&program_id, &accounts, &instruction_data);
        assert!(result.is_err());
    }
}
