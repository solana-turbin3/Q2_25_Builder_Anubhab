use anchor_lang::prelude::*;

declare_id!("22222222222222222222222222222222222222222222");

#[program]
pub mod vault {
    use anchor_lang::system_program::{transfer, Transfer};

    use super::*;

    pub fn deposit(ctx: Context<VaultTransfer>, amount: u64) -> Result<()> {
        require_gt!(
            amount,
            Rent::get()?.minimum_balance(0),
            VaultError::InvalidAmount
        );

        let transfer_acc = Transfer {
            from: ctx.accounts.signer.to_account_info(),
            to: ctx.accounts.vault.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(ctx.accounts.system_account.to_account_info(), transfer_acc);

        transfer(cpi_ctx, amount);

        Ok(())
    }

    pub fn withdraw(ctx: Context<VaultTransfer>) -> Result<()> {
        let bindings = ctx.accounts.signer.key();
        let signer_seeds = &[b"vault", bindings.as_ref(), &[ctx.bumps.vault]];

        let transfer_acc = Transfer {
            from: ctx.accounts.vault.to_account_info(),
            to: ctx.accounts.signer.to_account_info(),
        };

        let binding = [&signer_seeds[..]];
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.system_account.to_account_info(),
            transfer_acc,
            &binding,
        );

        transfer(cpi_ctx, ctx.accounts.vault.lamports())?;
        Ok(())
    }
}


#[derive(Accounts)] 
pub struct VaultTransfer<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds=[b"vault", signer.key().as_ref()],
        bump
    )]
    pub vault: SystemAccount<'info>,

    pub system_account: Program<'info, System>,
}

#[error_code]
pub enum VaultError {
    #[msg("More amount required")]
    InvalidAmount,
}
