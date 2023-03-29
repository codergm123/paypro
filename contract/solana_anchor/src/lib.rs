pub mod utils;
use borsh::{ BorshDeserialize, BorshSerialize };
use {
    crate::utils::*,
    anchor_lang::{
        prelude::*,
        AnchorDeserialize,
        AnchorSerialize,
        Key,
        solana_program::{
            program_pack::Pack,
            msg,
            system_instruction,
            system_program,
            program::{ invoke, invoke_signed },
            // nonce,
        },
    },
    spl_token::state,
};
declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

const PREFIX: &str = "offermaker";                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                

#[program]
pub mod solana_anchor {
	use super::*;

	pub fn init_monkpay(
		ctx : Context<InitMonkpay>,
		_bump : u8,
		_amount : u64
		) -> ProgramResult{
			let signer = &ctx.accounts.signer;
			let vault = &ctx.accounts.vault;
			let nonce_account = &ctx.accounts.nonce_account;
			let system_program = &ctx.accounts.system_program;
			let rent = &ctx.accounts.rent;
			let recent_blockhash = &ctx.accounts.recent_blockhash;
			let monkpay_data = &mut ctx.accounts.monkpay_data;
			let vec_instructions = system_instruction::create_nonce_account(
				signer.key,
				nonce_account.key,
				vault.key,
				_amount);
			for instruction in vec_instructions {
				invoke(
					&instruction,
					&[
						signer.clone(),
						nonce_account.clone(),
						system_program.to_account_info(),
						rent.clone(),
						recent_blockhash.clone(),
					]
				)?;
			}

			monkpay_data.owner = signer.key();
			monkpay_data.nonce_account = nonce_account.key();
			monkpay_data.bump = _bump;
			monkpay_data.usdt_account = *ctx.accounts.usdt_account.key;
			monkpay_data.usdt_mint = *ctx.accounts.usdt_mint.key;
			monkpay_data.total_accounts =0;

			Ok(())

	}

	pub fn create_account(
		ctx: Context<CreateAccount>,
		) -> ProgramResult {
		msg!("Init Account on MonkPay");

		let account_data = &mut ctx.accounts.account_data;
		let monkpay_data = &mut ctx.accounts.monkpay_data;

		account_data.sol_amount = 0;
		account_data.usdt_amount = 0;
		account_data.monkpay_data = monkpay_data.key();
		account_data.owner = *ctx.accounts.owner.key;

		monkpay_data.total_accounts += 1;

		Ok(())
	}

	pub fn deposit_usdt(
		ctx:Context<DepositUSDT>,
		_amount:u64
		) -> ProgramResult {
		msg!("Deposit USDT");

		let account_data = &mut ctx.accounts.account_data;
		let monkpay_data = &ctx.accounts.monkpay_data;
		let usdt_mint : state::Mint = state::Mint::unpack_from_slice(&ctx.accounts.usdt_mint.data.borrow())?;
		let source_account : state::Account = state::Account::unpack_from_slice(&ctx.accounts.source_account.data.borrow())?;
		let destination_account : state::Account = state::Account::unpack_from_slice(&ctx.accounts.destination_account.data.borrow())?;

		if account_data.owner != *ctx.accounts.owner.key {
			msg!("You are not owner of this account");
			return Err(MonkError::InvalidOwner.into());
		}
		if source_account.mint != *ctx.accounts.usdt_mint.key {
			msg!("No match mint address");
			return Err(MonkError::InvalidMint.into());
		}
		if source_account.owner != *ctx.accounts.owner.key {
			msg!("You are not owner of this account");
			return Err(MonkError::InvalidOwner.into());
		}
		if monkpay_data.usdt_account != *ctx.accounts.destination_account.key {
			msg!("No match Destination address");
			return Err(MonkError::InvalidDestinationAccount.into());
		}

		spl_token_transfer_without_seed(
			TokenTransferParamsWithoutSeed{
				source: ctx.accounts.source_account.clone(),
				destination: ctx.accounts.destination_account.clone(),
				authority: ctx.accounts.owner.clone(),
				token_program: ctx.accounts.token_program.clone(),
				amount: _amount
			}
		)?;

		account_data.usdt_amount += _amount;
		Ok(())
	}

	pub fn withdraw_usdt(
		ctx: Context<WithdrawUSDT>,
		_amount: u64
		) -> ProgramResult {
		msg!("Withdraw USDT");

		let account_data = &mut ctx.accounts.account_data;
		let monkpay_data = &ctx.accounts.monkpay_data;
		let usdt_mint : state::Mint = state::Mint::unpack_from_slice(&ctx.accounts.usdt_mint.data.borrow())?;
		let source_account : state::Account = state::Account::unpack_from_slice(&ctx.accounts.source_account.data.borrow())?;
		let destination_account : state::Account = state::Account::unpack_from_slice(&ctx.accounts.destination_account.data.borrow())?;
		let owner = &ctx.accounts.owner;

		if account_data.owner != *ctx.accounts.owner.key {
			msg!("You are not owner of this account");
			return Err(MonkError::InvalidOwner.into());
		}
		if monkpay_data.usdt_account != *ctx.accounts.source_account.key {
			msg!("No match Source address");
			return Err(MonkError::InvalidSourceAccount.into());
		}
		if destination_account.owner != *ctx.accounts.owner.key {
			msg!("You are not owner of this account");
			return Err(MonkError::InvalidOwner.into());
		}
		if _amount > account_data.usdt_amount {
			msg!("You don't have enough amount to withdraw");
			return Err(MonkError::InvalidAmount.into());
		}

		let authority_seeds = &[PREFIX.as_bytes(), &[monkpay_data.bump]];

		spl_token_transfer(
					TokenTransferParams{
						source: ctx.accounts.source_account.clone(),
						destination: ctx.accounts.destination_account.clone(),
						authority: ctx.accounts.vault.clone(),
						authority_signer_seeds: authority_seeds,
						token_program: ctx.accounts.token_program.clone(),
						amount: _amount
					}
				)?;

		account_data.usdt_amount -= _amount;
		Ok(())

	}

	pub fn transfer_usdt(
		ctx: Context<TransferUSDT>,
		_amount: u64
		) -> ProgramResult {

		msg!("Transfer USDT");

		let source_account_data = &mut ctx.accounts.source_account_data;
		let destination_account_data = &mut ctx.accounts.destination_account_data;
		let owner = &ctx.accounts.owner;

		if owner.key() != source_account_data.owner {
			msg!("You are not owner of this account");
			return Err(MonkError::InvalidOwner.into());
		}
		if _amount > source_account_data.usdt_amount {
			msg!("You don't have enough amount to withdraw");
			return Err(MonkError::InvalidAmount.into());
		}

		source_account_data.usdt_amount -= _amount;
		destination_account_data.usdt_amount += _amount;
		Ok(())
	}	

	pub fn refund_usdt(
		ctx: Context<RefundUSDT>,
		_amount: u64
		) -> ProgramResult {
		msg!("Refund all USDT!");

		let monkpay_data = &ctx.accounts.monkpay_data;
		let source_account : state::Account = state::Account::unpack_from_slice(&ctx.accounts.source_account.data.borrow())?;
		let destination_account : state::Account = state::Account::unpack_from_slice(&ctx.accounts.destination_account.data.borrow())?;
		let owner = &ctx.accounts.owner;

		if monkpay_data.owner != *ctx.accounts.owner.key {
			msg!("You are not owner of this account");
			return Err(MonkError::InvalidOwner.into());
		}
		if monkpay_data.usdt_account != *ctx.accounts.source_account.key {
			msg!("No match Source address");
			return Err(MonkError::InvalidSourceAccount.into());
		}
		if destination_account.owner != *ctx.accounts.owner.key {
			msg!("You are not owner of this account");
			return Err(MonkError::InvalidOwner.into());
		}

		let authority_seeds = &[PREFIX.as_bytes(), &[monkpay_data.bump]];

		spl_token_transfer(
					TokenTransferParams{
						source: ctx.accounts.source_account.clone(),
						destination: ctx.accounts.destination_account.clone(),
						authority: ctx.accounts.vault.clone(),
						authority_signer_seeds: authority_seeds,
						token_program: ctx.accounts.token_program.clone(),
						amount: _amount
					}
				)?;
		Ok(())
	}
}

#[derive(Accounts)]
#[instruction(_bump: u8)]
pub struct InitMonkpay<'info> {
	#[account(mut, signer)]
	signer: AccountInfo<'info>,

	#[account(init, payer=signer, space=8 + MONKPAY_SIZE)]
	monkpay_data: ProgramAccount<'info, IMonkpayData>,

	#[account(mut)]
	nonce_account: AccountInfo<'info>,

	#[account(owner=spl_token::id())]
	usdt_account: AccountInfo<'info>,

	#[account(owner=spl_token::id())]
	usdt_mint: AccountInfo<'info>,

	#[account(seeds=[PREFIX.as_bytes()],bump=_bump,)]
	vault: AccountInfo<'info>,

	system_program: Program<'info, System>,

	rent: AccountInfo<'info>,

	recent_blockhash: AccountInfo<'info>,

	rand: AccountInfo<'info>,

}

#[derive(Accounts)]
pub struct CreateAccount<'info> {
	#[account(mut, signer)]
	owner: AccountInfo<'info>,

	#[account(mut)]
	monkpay_data: ProgramAccount<'info, IMonkpayData>,

	#[account(init, payer = owner, space = 8 + ACCOUNT_DATA)]
	account_data: ProgramAccount<'info, IAccountData>,

	system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct DepositUSDT<'info> {
	#[account(mut, signer)]
	owner: AccountInfo<'info>,

	#[account(mut)]
	account_data: ProgramAccount<'info, IAccountData>,

	monkpay_data: ProgramAccount<'info, IMonkpayData>,

	#[account(owner = spl_token::id())]
	usdt_mint: AccountInfo<'info>,

	#[account(mut, owner=spl_token::id())]
	source_account: AccountInfo<'info>,

	#[account(mut, owner=spl_token::id())]
	destination_account: AccountInfo<'info>,

	#[account(address=spl_token::id())]
	token_program : AccountInfo<'info>,
}



#[derive(Accounts)]
pub struct WithdrawUSDT<'info> {
	#[account(mut, signer)]
	owner: AccountInfo<'info>,

	#[account(mut)]
	account_data: ProgramAccount<'info, IAccountData>,

	monkpay_data: ProgramAccount<'info, IMonkpayData>,

	vault: AccountInfo<'info>,

	#[account(mut, owner=spl_token::id())]
	source_account: AccountInfo<'info>,

	#[account(owner = spl_token::id())]
	usdt_mint: AccountInfo<'info>,

	#[account(mut, owner=spl_token::id())]
	destination_account: AccountInfo<'info>,

	#[account(address=spl_token::id())]
	token_program : AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct RefundUSDT<'info> {
	#[account(mut, signer)]
	owner: AccountInfo<'info>,

	monkpay_data: ProgramAccount<'info, IMonkpayData>,

	vault: AccountInfo<'info>,

	#[account(mut, owner=spl_token::id())]
	source_account: AccountInfo<'info>,

	#[account(mut, owner=spl_token::id())]
	destination_account: AccountInfo<'info>,

	#[account(address=spl_token::id())]
	token_program : AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct TransferUSDT<'info> {
	#[account(mut, signer)]
	owner: AccountInfo<'info>,

	#[account(mut)]
	source_account_data: ProgramAccount<'info, IAccountData>,

	#[account(mut)]
	destination_account_data: ProgramAccount<'info, IAccountData>,
}
pub const MONKPAY_SIZE: usize = 32 + 32 + 32 + 32 + 8 + 1;
pub const ACCOUNT_DATA: usize = 32 + 32 + 8 + 8;

#[account]
pub struct IMonkpayData {
	pub owner: Pubkey,
	pub nonce_account: Pubkey,
	pub bump: u8,
	pub usdt_account: Pubkey,
	pub usdt_mint: Pubkey,
	pub total_accounts: u64
}

#[account]
pub struct IAccountData {
	pub owner: Pubkey,
	pub sol_amount: u64,
	pub usdt_amount: u64,
	pub monkpay_data: Pubkey
}

#[error]
pub enum MonkError {
	#[msg("Invalid Owner")]
	InvalidOwner,

	#[msg("Invalid mint key")]
	InvalidMint,

	#[msg("Destination address dismatch with MonkPay")]
	InvalidDestinationAccount,

	#[msg("Source address dismatch with MonkPay")]
	InvalidSourceAccount,

	#[msg("Invalid Amount")]
	InvalidAmount,

	#[msg("Token mint to failed")]
    TokenMintToFailed,

    #[msg("Token set authority failed")]
    TokenSetAuthorityFailed,

    #[msg("Token transfer failed")]
    TokenTransferFailed,
}