use core::mem::{size_of, transmute};

use pinocchio::{
    account_info::{AccountInfo, Ref},
    entrypoint,
    instruction::{Seed, Signer},
    msg,
    program_error::ProgramError,
    pubkey::{find_program_address, Pubkey},
    sysvars::{
        clock::Clock,
        instructions::{Instructions, IntrospectedInstruction},
        Sysvar,
    },
    ProgramResult,
};
use pinocchio_secp256r1_instruction::{Secp256r1Instruction, Secp256r1Pubkey};
use pinocchio_system::instructions::Transfer;

entrypoint!(process_instruction);
// nostd_panic_handler!();

pub const ID: Pubkey = [
    0x0f, 0x1e, 0x6b, 0x14, 0x21, 0xc0, 0x4a, 0x07, 0x04, 0x31, 0x26, 0x5c, 0x19, 0xc5, 0xbb, 0xee,
    0x19, 0x92, 0xba, 0xe8, 0xaf, 0xd1, 0xcd, 0x07, 0x8e, 0xf8, 0xaf, 0x70, 0x47, 0xdc, 0x11, 0xf7,
];

fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    match instruction_data.split_first() {
        Some((Deposit::DISCRIMINATOR, data)) => Deposit::try_from((data, accounts))?.process(),
        Some((Withdraw::DISCRIMINATOR, data)) => Withdraw::try_from((data, accounts))?.process(),
        _ => Err(ProgramError::InvalidInstructionData),
    }
}

//Define struct for Accounts for deposit ix
pub struct DepositAccounts<'a> {
    pub payer: &'a AccountInfo,
    pub vault: &'a AccountInfo,
}

//Implement try from for DepositAccounts

impl<'a> TryFrom<&'a [AccountInfo]> for DepositAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        let [payer, vault, _] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        if !payer.is_signer() {
            return Err(ProgramError::InvalidAccountOwner);
        }

        if !payer.is_owned_by(&pinocchio_system::ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        Ok(Self { payer, vault })
    }
}

pub struct DepositIxData {
    pub sceppubkey: Secp256r1Pubkey,
    pub amount: u64,
}

impl<'a> TryFrom<&'a [u8]> for DepositIxData {
    type Error = ProgramError;

    // fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {

    //     let ix_data: Self = unsafe {
    //         transmute(
    //             TryInto::<[u8; size_of::<DepositIxData>()]>::try_into(data)
    //                 .map_err(|_| ProgramError::InvalidInstructionData)?,
    //         )
    //     };

    //     Ok(ix_data)
    // }

    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        if data.len() != 41 {
            return Err(ProgramError::InvalidInstructionData);
        }

        let pubkey = Secp256r1Pubkey::try_from(&data[..33])
            .map_err(|_| ProgramError::InvalidInstructionData)?;

        let amount = u64::from_le_bytes(data[33..41].try_into().unwrap());

        Ok(Self {
            sceppubkey: pubkey,
            amount,
        })
    }
}

//Let's combine accounts and ixs

pub struct Deposit<'a> {
    pub accounts: DepositAccounts<'a>,
    pub ix_data: DepositIxData,
}

impl<'a> TryFrom<(&'a [u8], &'a [AccountInfo])> for Deposit<'a> {
    type Error = ProgramError;

    fn try_from((data, accounts): (&'a [u8], &'a [AccountInfo])) -> Result<Self, Self::Error> {
        let accounts = DepositAccounts::try_from(accounts)?;
        let instruction_data = DepositIxData::try_from(data)?;

        Ok(Self {
            accounts,
            ix_data: instruction_data,
        })
    }
}

impl<'a> Deposit<'a> {
    pub const DISCRIMINATOR: &'a u8 = &0;

    pub fn process(&mut self) -> ProgramResult {
        let seeds = [
            b"vault",
            &self.ix_data.sceppubkey[..1],
            &self.ix_data.sceppubkey[1..33],
        ];
        let (vault_key, _) = find_program_address(&seeds, &crate::ID);

        if vault_key != *self.accounts.vault.key() {
            return Err(ProgramError::InvalidAccountOwner);
        }

        if vault_key != *self.accounts.vault.key() {
            msg!("Line 118");
            return Err(ProgramError::InvalidAccountOwner);
        }

        Transfer {
            from: self.accounts.payer,
            to: self.accounts.vault,
            lamports: self.ix_data.amount,
        }
        .invoke()?;
        Ok(())
    }
}

pub struct WithdrawAccounts<'a> {
    pub payer: &'a AccountInfo,
    pub vault: &'a AccountInfo,
    pub ix: &'a AccountInfo,
}

impl<'a> TryFrom<&'a [AccountInfo]> for WithdrawAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        let [payer, vault, ix, _system_program] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        if !vault.is_owned_by(&pinocchio_system::ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        if vault.lamports().eq(&0) {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(Self { payer, vault, ix })
    }
}

pub struct WithdrawIxData {
    pub bump: [u8; 1],
}

impl<'a> TryFrom<&'a [u8]> for WithdrawIxData {
    type Error = ProgramError;

    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        Ok(unsafe {
            transmute(
                TryInto::<[u8; size_of::<WithdrawIxData>()]>::try_into(data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?,
            )
        })
    }
}

pub struct Withdraw<'a> {
    pub accounts: WithdrawAccounts<'a>,
    pub ix_data: WithdrawIxData,
}

impl<'a> TryFrom<(&'a [u8], &'a [AccountInfo])> for Withdraw<'a> {
    type Error = ProgramError;

    fn try_from((data, accounts): (&'a [u8], &'a [AccountInfo])) -> Result<Self, Self::Error> {
        let accounts = WithdrawAccounts::try_from(accounts)?;
        let ix_data = WithdrawIxData::try_from(data)?;

        Ok(Self { accounts, ix_data })
    }
}

impl<'a> Withdraw<'a> {
    pub const DISCRIMINATOR: &'a u8 = &1;

    pub fn process(&mut self) -> ProgramResult {
        let ix: Instructions<Ref<[u8]>> = Instructions::try_from(self.accounts.ix)?;

        let iix: IntrospectedInstruction = ix.get_instruction_relative(1)?;

        let secp256r1_ix = Secp256r1Instruction::try_from(&iix)?;

        if secp256r1_ix.num_signatures() != 1 {
            return Err(ProgramError::InvalidInstructionData);
        }

        let signer: Secp256r1Pubkey = *secp256r1_ix.get_signer(0)?;

        // let (payer, expiry) = secp256r1_ix
        //     .get_message_data(0)?
        //     .split_at_checked(32)
        //     .ok_or(ProgramError::InvalidInstructionData)?;
        let msg = secp256r1_ix.get_message_data(0)?;
        if msg.len() < 40 {
            return Err(ProgramError::InvalidInstructionData);
        }
        let payer = &msg[..32];
        let expiry_bytes = &msg[32..40];
        let expiry = i64::from_le_bytes(
            expiry_bytes
                .try_into()
                .map_err(|_| ProgramError::InvalidInstructionData)?,
        );

        if self.accounts.payer.key().ne(payer) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        let now = Clock::get()?.unix_timestamp;

        // let expiry = i64::from_le_bytes(
        //     expiry
        //         .try_into()
        //         .map_err(|_| ProgramError::InvalidInstructionData)?,
        // );

        if now > expiry {
            return Err(ProgramError::InvalidInstructionData);
        }

        let seeds = [
            Seed::from(b"vault"),
            Seed::from(signer[..1].as_ref()),
            Seed::from(signer[1..].as_ref()),
            Seed::from(&self.ix_data.bump),
        ];
        let signers = [Signer::from(&seeds)];

        Transfer {
            from: self.accounts.vault,
            to: self.accounts.payer,
            lamports: self.accounts.vault.lamports(),
        }
        .invoke_signed(&signers)
    }
}
