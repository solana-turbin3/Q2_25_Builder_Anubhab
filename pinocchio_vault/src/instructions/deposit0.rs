use core::mem::size_of;

use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, pubkey::find_program_address,
    ProgramResult,
};
use pinocchio_system::instructions::Transfer;


//Accounts Structure
pub struct DepositAccounts<'info> {
    pub owner: &'info AccountInfo,
    pub vault: &'info AccountInfo,
}


//Implementing TryFrom for Accounts
impl<'info> TryFrom<&'info [AccountInfo]> for DepositAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountInfo]) -> Result<Self, Self::Error> {
        let [owner, vault, _] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        //Account checks
        if !owner.is_signer() {
            return Err(ProgramError::InvalidAccountOwner);
        }

        if !vault.is_owned_by(&pinocchio_system::ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        if vault.lamports().ne(&0) {
            return Err(ProgramError::InvalidAccountData);
        }

        let (vault_key, _) = find_program_address(&[b"vault", owner.key()], &crate::ID);
        if vault.key().ne(&vault_key) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        Ok(Self { owner, vault })
    }
}


//Instructions Data structure
pub struct DepositInstructionData {
    pub amount: u64,
}


//Implementing TryFrom for instructions data 
impl<'info> TryFrom<&'info [u8]> for DepositInstructionData {
    type Error = ProgramError;

    fn try_from(data: &'info [u8]) -> Result<Self, Self::Error> {
        if data.len() != size_of::<u64>() {
            return Err(ProgramError::InvalidInstructionData);
        }

        let amount = u64::from_le_bytes(data.try_into().unwrap());

        if amount.eq(&0) {
            return Err(ProgramError::InvalidInstructionData);
        }

        Ok(Self { amount })
    }
}

//Finally the deposit struct that combines both the accounts and instructions data
pub struct Deposit<'info> {
    pub accounts: DepositAccounts<'info>,
    pub instruction_datas: DepositInstructionData,
}


//Implementing TryFrom for Deposit struct
impl<'info> TryFrom<(&'info [u8], &'info [AccountInfo])> for Deposit<'info> {
    type Error = ProgramError;

    fn try_from(
        (data, accounts): (&'info [u8], &'info [AccountInfo]),
    ) -> Result<Self, Self::Error> {
        let accounts = DepositAccounts::try_from(accounts)?;
        let instruction_datas: DepositInstructionData = DepositInstructionData::try_from(data)?;

        Ok(Self {
            accounts,
            instruction_datas,
        })
    }
}


//Finally writing the logic for deposit
impl<'info> Deposit<'info> {
    pub const DISCRIMINATOR: &'info u8 = &0;

    pub fn process(&mut self) -> ProgramResult {
        Transfer {
            from: self.accounts.owner,
            to: self.accounts.vault,
            lamports: self.instruction_datas.amount,
        }
        .invoke()?;

        Ok(())
    }
}
