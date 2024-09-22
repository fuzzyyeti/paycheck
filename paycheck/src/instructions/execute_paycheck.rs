use crate::consts::PAYCHECK_SEED;
use crate::error::PaycheckProgramError;
use crate::processor::PaycheckInstructions;
use crate::state::Paycheck;
use crate::ID;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::account_info::{next_account_info, AccountInfo};
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::msg;
use solana_program::program::{invoke, invoke_signed};
use solana_program::program_error::ProgramError;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use solana_program::rent::Rent;
use solana_program::sysvar::Sysvar;
use spl_token::state::Account;
use whirlpools_state::Whirlpool;

#[derive(Debug, BorshDeserialize, BorshSerialize)]
pub struct ExecutePaycheckArgs {
    pub creator: Pubkey,
}
pub fn process_execute_paycheck(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: ExecutePaycheckArgs,
) -> Result<(), ProgramError> {
    let account_info_iter = &mut accounts.iter();
    let payer = next_account_info(account_info_iter)?;
    let paycheck_account = next_account_info(account_info_iter)?;
    let whirlpool = next_account_info(account_info_iter)?;
    let target_mint = next_account_info(account_info_iter)?;
    let creator_ata = next_account_info(account_info_iter)?;
    let token_vault_a = next_account_info(account_info_iter)?;
    let receiver_ata = next_account_info(account_info_iter)?;
    let payer_ata = next_account_info(account_info_iter)?;
    let temp_token_account = next_account_info(account_info_iter)?;
    let token_vault_b = next_account_info(account_info_iter)?;
    let tick_array_0 = next_account_info(account_info_iter)?;
    let tick_array_1 = next_account_info(account_info_iter)?;
    let tick_array_2 = next_account_info(account_info_iter)?;
    let oracle = next_account_info(account_info_iter)?;
    msg!("1");
    let whirlpool_data = Whirlpool::try_from_slice(&whirlpool.data.borrow())?;
    msg!("2");
    let paycheck_data = Paycheck::try_from_slice(&paycheck_account.data.borrow())?;
    msg!("3");
    let required_lamports = Rent::get()?.minimum_balance(Account::LEN);
    msg!("creating account ix");
    let init_account_ix = solana_program::system_instruction::create_account(
        payer.key,
        temp_token_account.key,
        required_lamports,
        Account::LEN as u64,
        &spl_token::id(),
    );
    msg!("invoking create acct");

    invoke(
        &init_account_ix,
        &[payer.clone(), temp_token_account.clone()],
    )?;

    msg!("the token program id is {:?}", spl_token::id());
    msg!("temp_account owner is {:?}", temp_token_account.owner);
    msg!("mint owner is {:?}", target_mint.owner);
    let create_account_ix = spl_token::instruction::initialize_account3(
        &spl_token::id(),
        temp_token_account.key,
        &target_mint.key,
        &paycheck_account.key,
    )?;

    invoke_signed(
        &create_account_ix,
        &[temp_token_account.clone(), target_mint.clone()],
        &[&[
            PAYCHECK_SEED,
            &whirlpool.key.to_bytes(),
            &args.creator.to_bytes(),
            &[paycheck_data.bump],
        ]],
    )?;

    Ok(())
}

pub fn execute_paycheck_ix(
    payer: Pubkey,
    creator: Pubkey,
    whirlpool: Pubkey,
    target_mint: Pubkey,
    temp_token_account: Pubkey,
) -> Result<Instruction, PaycheckProgramError> {
    let paycheck = Pubkey::find_program_address(
        &[PAYCHECK_SEED, &whirlpool.to_bytes(), &creator.to_bytes()],
        &ID,
    )
    .0;
    println!("Paycheck address 2 {:?}", paycheck);

    let data = borsh::to_vec(&PaycheckInstructions::ExecutePaycheck(
        ExecutePaycheckArgs { creator },
    ))
    .map_err(|_| PaycheckProgramError::InvalidInstructionData)?;
    println!("Ix DATA {:?}", data);
    let paycheck_instruction_des = PaycheckInstructions::try_from_slice(&data);
    println!("PID {:?}", paycheck_instruction_des);


    let creator_ata =
        spl_associated_token_account::get_associated_token_address(&creator, &target_mint);

    let payer_ata = Pubkey::new_unique();
    let token_vault_a = Pubkey::new_unique();
    let token_vault_b = Pubkey::new_unique();
    let receiver_ata = Pubkey::new_unique();
    let tick_array_0 = Pubkey::new_unique();
    let tick_array_1 = Pubkey::new_unique();
    let tick_array_2 = Pubkey::new_unique();
    let oracle = Pubkey::new_unique();

    Ok(Instruction {
        program_id: ID,
        accounts: vec![
            AccountMeta::new(payer, true),
            AccountMeta::new(paycheck, false),
            AccountMeta::new(whirlpool, false),
            AccountMeta::new(target_mint, false),
            AccountMeta::new(creator_ata, false),
            AccountMeta::new(token_vault_a, false),
            AccountMeta::new(receiver_ata, false),
            AccountMeta::new(payer_ata, false),
            AccountMeta::new(temp_token_account, true),
            AccountMeta::new(token_vault_b, false),
            AccountMeta::new(tick_array_0, false),
            AccountMeta::new(tick_array_1, false),
            AccountMeta::new(tick_array_2, false),
            AccountMeta::new_readonly(oracle, false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(whirlpools_state::ID, false),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
        ],
        data,
    })
}
