use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};
use borsh::{BorshDeserialize, BorshSerialize};

/// Define the counter account structure
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct CounterAccount {
    pub is_initialized: bool,
    pub count: u64,
    pub owner: Pubkey,
}

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = CounterInstruction::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    match instruction {
        CounterInstruction::Initialize => initialize(program_id, accounts),
        CounterInstruction::Increment => increment(program_id, accounts),
        CounterInstruction::Decrement => decrement(program_id, accounts),
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum CounterInstruction {
    /// Initialize counter
    /// Accounts: [writable] counter account, [signer] owner
    Initialize,
    /// Increment counter by 1
    /// Accounts: [writable] counter account, [signer] owner
    Increment,
    /// Decrement counter by 1
    /// Accounts: [writable] counter account, [signer] owner
    Decrement,
}

fn initialize(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let counter_account = next_account_info(accounts_iter)?;
    let owner = next_account_info(accounts_iter)?;

    if counter_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    if !counter_account.is_writable {
        return Err(ProgramError::InvalidAccountData);
    }

    if !owner.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut counter_data = CounterAccount::try_from_slice(&counter_account.data.borrow())
        .unwrap_or(CounterAccount {
            is_initialized: false,
            count: 0,
            owner: Pubkey::default(),
        });

    if counter_data.is_initialized {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    counter_data.is_initialized = true;
    counter_data.count = 0;
    counter_data.owner = *owner.key;

    counter_data.serialize(&mut &mut counter_account.data.borrow_mut()[..])?;
    msg!("Counter initialized by {}", owner.key);

    Ok(())
}

fn increment(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let counter_account = next_account_info(accounts_iter)?;
    let signer = next_account_info(accounts_iter)?;

    if counter_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    if !counter_account.is_writable {
        return Err(ProgramError::InvalidAccountData);
    }

    if !signer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut counter_data = CounterAccount::try_from_slice(&counter_account.data.borrow())?;

    if !counter_data.is_initialized {
        return Err(ProgramError::UninitializedAccount);
    }

    if counter_data.owner != *signer.key {
        msg!("Only owner can increment");
        return Err(ProgramError::InvalidAccountData);
    }

    counter_data.count = counter_data
        .count
        .checked_add(1)
        .ok_or(ProgramError::InvalidInstructionData)?;

    counter_data.serialize(&mut &mut counter_account.data.borrow_mut()[..])?;
    msg!("Counter incremented to {}", counter_data.count);

    Ok(())
}

fn decrement(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let counter_account = next_account_info(accounts_iter)?;
    let signer = next_account_info(accounts_iter)?;

    if counter_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    if !counter_account.is_writable {
        return Err(ProgramError::InvalidAccountData);
    }

    if !signer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut counter_data = CounterAccount::try_from_slice(&counter_account.data.borrow())?;

    if !counter_data.is_initialized {
        return Err(ProgramError::UninitializedAccount);
    }

    if counter_data.owner != *signer.key {
        msg!("Only owner can decrement");
        return Err(ProgramError::InvalidAccountData);
    }

    counter_data.count = counter_data
        .count
        .checked_sub(1)
        .ok_or(ProgramError::InvalidInstructionData)?;

    counter_data.serialize(&mut &mut counter_account.data.borrow_mut()[..])?;
    msg!("Counter decremented to {}", counter_data.count);

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use solana_program::clock::Epoch;

    #[test]
    fn test_initialize() {
        let program_id = Pubkey::default();
        let key = Pubkey::default();
        let owner_key = Pubkey::new_unique();
        let mut lamports = 0;
        let mut data = vec![0; 100];

        let counter_account = AccountInfo::new(
            &key,
            false,
            true,
            &mut lamports,
            &mut data,
            &program_id,
            false,
            Epoch::default(),
        );

        let mut owner_lamports = 0;
        let mut owner_data = vec![];
        let owner_account = AccountInfo::new(
            &owner_key,
            true,
            false,
            &mut owner_lamports,
            &mut owner_data,
            &program_id,
            false,
            Epoch::default(),
        );

        let accounts = vec![counter_account, owner_account];
        let instruction_data = CounterInstruction::Initialize.try_to_vec().unwrap();

        assert!(process_instruction(&program_id, &accounts, &instruction_data).is_ok());
    }

    #[test]
    fn test_increment_overflow_protection() {
        let program_id = Pubkey::default();
        let key = Pubkey::default();
        let owner_key = Pubkey::new_unique();
        let mut lamports = 0;

        let counter_data = CounterAccount {
            is_initialized: true,
            count: u64::MAX,
            owner: owner_key,
        };
        let mut data = counter_data.try_to_vec().unwrap();
        data.resize(100, 0);

        let counter_account = AccountInfo::new(
            &key,
            false,
            true,
            &mut lamports,
            &mut data,
            &program_id,
            false,
            Epoch::default(),
        );

        let mut owner_lamports = 0;
        let mut owner_data = vec![];
        let owner_account = AccountInfo::new(
            &owner_key,
            true,
            false,
            &mut owner_lamports,
            &mut owner_data,
            &program_id,
            false,
            Epoch::default(),
        );

        let accounts = vec![counter_account, owner_account];
        let instruction_data = CounterInstruction::Increment.try_to_vec().unwrap();

        assert!(process_instruction(&program_id, &accounts, &instruction_data).is_err());
    }
}
