use anchor_lang::prelude::*;

declare_id!("22222222222222222222222222222222222222222222");

#[program]
pub mod vault {

    use anchor_lang::system_program::{transfer, Transfer};

    use crate::program::Vault;

    use super::*;

    //2 functions

    //Deposit function

    pub fn deposit(ctx: Context<VaultContext>, amount: u64) -> Result<()> {
        let transfer_acc = Transfer {
            from: ctx.accounts.signer.to_account_info(),
            to: ctx.accounts.vault.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(ctx.accounts.system_account.to_account_info(), transfer_acc);

        transfer(cpi_ctx, amount)?;
        Ok(())
    }

    //Withdraw function

    pub fn withdraw(ctx: Context<VaultContext>) -> Result<()> {
        let transfer_acc = Transfer {
            from: ctx.accounts.vault.to_account_info(),
            to: ctx.accounts.signer.to_account_info(),
        };

        let signer_key = ctx.accounts.signer.key();

        let signer_seeds = &[b"vault", signer_key.as_ref(), &[ctx.bumps.vault]];

        let signer_bind = [&signer_seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.system_account.to_account_info(),
            transfer_acc,
            &signer_bind,
        );

        transfer(cpi_ctx, ctx.accounts.vault.lamports())?;
        Ok(())
    }
}

//Deposit and withdraw function context

#[derive(Accounts)]
pub struct VaultContext<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [b"vault", signer.key().as_ref()],
        bump
    )]
    pub vault: SystemAccount<'info>,
    pub system_account: Program<'info, System>,
}
