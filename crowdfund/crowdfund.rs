use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program::invoke,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction,
    sysvar::Sysvar,
};
use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct CrowdfundAccount {
    pub is_initialized: bool,
    pub owner: Pubkey,
    pub goal: u64,
    pub deadline: u64, // slot number
    pub total_raised: u64,
    pub finalized: bool,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct ContributorAccount {
    pub amount: u64,
}

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = CrowdfundInstruction::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    match instruction {
        CrowdfundInstruction::Initialize { goal, duration_slots } => {
            initialize(program_id, accounts, goal, duration_slots)
        }
        CrowdfundInstruction::Contribute { amount } => {
            contribute(program_id, accounts, amount)
        }
        CrowdfundInstruction::Withdraw => withdraw(program_id, accounts),
        CrowdfundInstruction::Refund => refund(program_id, accounts),
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum CrowdfundInstruction {
    /// Initialize crowdfund campaign
    /// Accounts: [writable] campaign, [signer] owner, [] system_program
    Initialize { goal: u64, duration_slots: u64 },
    /// Contribute funds
    /// Accounts: [writable] campaign, [writable] contributor_record, [writable, signer] contributor, [] system_program
    Contribute { amount: u64 },
    /// Withdraw funds if successful (owner only)
    /// Accounts: [writable] campaign, [writable] owner, [] system_program
    Withdraw,
    /// Refund contribution if failed
    /// Accounts: [writable] campaign, [writable] contributor_record, [writable] contributor, [] system_program
    Refund,
}

fn initialize(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    goal: u64,
    duration_slots: u64,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let campaign_account = next_account_info(accounts_iter)?;
    let owner = next_account_info(accounts_iter)?;

    if campaign_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    if !owner.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if goal == 0 {
        msg!("Goal must be greater than zero");
        return Err(ProgramError::InvalidInstructionData);
    }

    if duration_slots == 0 {
        msg!("Duration must be greater than zero");
        return Err(ProgramError::InvalidInstructionData);
    }

    let clock = Clock::get()?;
    let deadline = clock.slot + duration_slots;

    let campaign = CrowdfundAccount {
        is_initialized: true,
        owner: *owner.key,
        goal,
        deadline,
        total_raised: 0,
        finalized: false,
    };

    campaign.serialize(&mut &mut campaign_account.data.borrow_mut()[..])?;
    msg!("Crowdfund initialized: goal={}, deadline={}", goal, deadline);

    Ok(())
}

fn contribute(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let campaign_account = next_account_info(accounts_iter)?;
    let contributor_record = next_account_info(accounts_iter)?;
    let contributor = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;

    if campaign_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    if !contributor.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if amount == 0 {
        msg!("Must contribute a positive amount");
        return Err(ProgramError::InvalidInstructionData);
    }

    let mut campaign = CrowdfundAccount::try_from_slice(&campaign_account.data.borrow())?;

    if !campaign.is_initialized {
        return Err(ProgramError::UninitializedAccount);
    }

    let clock = Clock::get()?;
    if clock.slot >= campaign.deadline {
        msg!("Campaign has ended");
        return Err(ProgramError::InvalidInstructionData);
    }

    if campaign.finalized {
        msg!("Campaign already finalized");
        return Err(ProgramError::InvalidAccountData);
    }

    // Transfer lamports from contributor to campaign account
    invoke(
        &system_instruction::transfer(contributor.key, campaign_account.key, amount),
        &[contributor.clone(), campaign_account.clone(), system_program.clone()],
    )?;

    // Update or create contributor record
    let mut contributor_data = if contributor_record.data_len() > 0 {
        ContributorAccount::try_from_slice(&contributor_record.data.borrow())
            .unwrap_or(ContributorAccount { amount: 0 })
    } else {
        ContributorAccount { amount: 0 }
    };

    contributor_data.amount = contributor_data
        .amount
        .checked_add(amount)
        .ok_or(ProgramError::InvalidInstructionData)?;

    if contributor_record.owner == program_id {
        contributor_data.serialize(&mut &mut contributor_record.data.borrow_mut()[..])?;
    }

    campaign.total_raised = campaign
        .total_raised
        .checked_add(amount)
        .ok_or(ProgramError::InvalidInstructionData)?;

    campaign.serialize(&mut &mut campaign_account.data.borrow_mut()[..])?;

    msg!("Contributed {} lamports. Total raised: {}", amount, campaign.total_raised);

    if campaign.total_raised >= campaign.goal {
        msg!("Goal reached!");
    }

    Ok(())
}

fn withdraw(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let campaign_account = next_account_info(accounts_iter)?;
    let owner = next_account_info(accounts_iter)?;

    if campaign_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    if !owner.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut campaign = CrowdfundAccount::try_from_slice(&campaign_account.data.borrow())?;

    if campaign.owner != *owner.key {
        msg!("Only owner can withdraw");
        return Err(ProgramError::InvalidAccountData);
    }

    let clock = Clock::get()?;
    if clock.slot < campaign.deadline {
        msg!("Campaign still active");
        return Err(ProgramError::InvalidInstructionData);
    }

    if campaign.finalized {
        msg!("Already finalized");
        return Err(ProgramError::InvalidAccountData);
    }

    if campaign.total_raised < campaign.goal {
        msg!("Goal not reached");
        return Err(ProgramError::InvalidInstructionData);
    }

    campaign.finalized = true;
    let amount = campaign.total_raised;

    // Transfer funds to owner
    **campaign_account.try_borrow_mut_lamports()? -= amount;
    **owner.try_borrow_mut_lamports()? += amount;

    campaign.serialize(&mut &mut campaign_account.data.borrow_mut()[..])?;
    msg!("Withdrawn {} lamports", amount);

    Ok(())
}

fn refund(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let campaign_account = next_account_info(accounts_iter)?;
    let contributor_record = next_account_info(accounts_iter)?;
    let contributor = next_account_info(accounts_iter)?;

    if campaign_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    if !contributor.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let campaign = CrowdfundAccount::try_from_slice(&campaign_account.data.borrow())?;

    let clock = Clock::get()?;
    if clock.slot < campaign.deadline {
        msg!("Campaign still active");
        return Err(ProgramError::InvalidInstructionData);
    }

    if campaign.total_raised >= campaign.goal {
        msg!("Goal was reached, no refunds");
        return Err(ProgramError::InvalidInstructionData);
    }

    if contributor_record.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    let mut contributor_data = ContributorAccount::try_from_slice(&contributor_record.data.borrow())?;

    if contributor_data.amount == 0 {
        msg!("No contribution to refund");
        return Err(ProgramError::InvalidAccountData);
    }

    let amount = contributor_data.amount;
    contributor_data.amount = 0;

    // Transfer lamports back to contributor
    **campaign_account.try_borrow_mut_lamports()? -= amount;
    **contributor.try_borrow_mut_lamports()? += amount;

    contributor_data.serialize(&mut &mut contributor_record.data.borrow_mut()[..])?;
    msg!("Refunded {} lamports", amount);

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use solana_program::clock::Epoch;

    #[test]
    fn test_initialize() {
        let program_id = Pubkey::default();
        let campaign_key = Pubkey::default();
        let owner_key = Pubkey::new_unique();
        let mut campaign_lamports = 0;
        let mut campaign_data = vec![0; 200];

        let campaign_account = AccountInfo::new(
            &campaign_key,
            false,
            true,
            &mut campaign_lamports,
            &mut campaign_data,
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

        let accounts = vec![campaign_account, owner_account];
        let instruction = CrowdfundInstruction::Initialize {
            goal: 1000,
            duration_slots: 100,
        };
        let instruction_data = instruction.try_to_vec().unwrap();

        let result = process_instruction(&program_id, &accounts, &instruction_data);
        assert!(result.is_ok());
    }
}
