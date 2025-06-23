use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    program_error::ProgramError,
    pubkey::{find_program_address, Pubkey},
    ProgramResult,
};
use pinocchio_system::instructions::Transfer;

pub struct WithdrawAccounts<'info> {
    pub owner: &'info AccountInfo,
    pub vault: &'info AccountInfo,
    pub bumps: [u8; 1],
}

impl<'info> TryFrom<&'info [AccountInfo]> for WithdrawAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountInfo]) -> Result<Self, Self::Error> {
        let [owner, vault, _system_program] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        if !owner.is_signer() {
            return Err(ProgramError::InvalidAccountOwner);
        }

        let (vault_key, bump) = find_program_address(&[b"vault", owner.key().as_ref()], &crate::ID);

        if &vault_key != vault.key() {
            return Err(ProgramError::InvalidAccountOwner);
        }

        Ok(Self {
            owner,
            vault,
            bumps: [bump],
        })
    }
}

pub struct Withdraw<'info> {
    pub accounts: WithdrawAccounts<'info>,
}

impl<'info> TryFrom<&'info [AccountInfo]> for Withdraw<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountInfo]) -> Result<Self, Self::Error> {
        let accounts = WithdrawAccounts::try_from(accounts)?;

        Ok(Self { accounts })
    }
}

impl<'info> Withdraw<'info> {
    pub const DISCRIMINATOR: &'info u8 = &1;

    pub fn process(&mut self) -> ProgramResult {
        let seeds = [
            Seed::from(b"vault"),
            Seed::from(self.accounts.owner.key().as_ref()),
            Seed::from(&self.accounts.bumps),
        ];
        let signers = [Signer::from(&seeds)];
        Transfer {
            from: self.accounts.vault,
            to: self.accounts.owner,
            lamports: self.accounts.vault.lamports(),
        }
        .invoke_signed(&signers)?;
        Ok(())
    }
}
