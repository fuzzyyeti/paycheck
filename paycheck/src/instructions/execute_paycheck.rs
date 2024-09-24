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
use whirlpools_state::{SwapArgs, Whirlpool};

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
    let spl_token_program = next_account_info(account_info_iter)?;
    let whirlpool_data = Whirlpool::try_from_slice(&whirlpool.data.borrow())?;
    let paycheck_data: Paycheck = Paycheck::try_from_slice(&paycheck_account.data.borrow())?;
    let required_lamports = Rent::get()?.minimum_balance(Account::LEN);

    // Create a temp token account to hold the swap output
    let init_account_ix = solana_program::system_instruction::create_account(
        payer.key,
        temp_token_account.key,
        required_lamports,
        Account::LEN as u64,
        &spl_token::id(),
    );

    invoke(
        &init_account_ix,
        &[payer.clone(), temp_token_account.clone()],
    )?;

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

    // Perform the swap
    let amount = paycheck_data.tip + paycheck_data.amount;
    let swap_discriminator: [u8; 8] = [248, 198, 158, 145, 225, 117, 135, 200];
    let input_args = SwapArgs {
        swap_discriminator,
        amount,
        other_amount_threshold: 0,
        sqrt_price_limit: 0,
        amount_specified_is_input: true,
        a_to_b: true,
    };

    let swap_ix = Instruction::new_with_borsh(
        whirlpools_state::ID,
        &input_args,
        vec![
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new(*paycheck_account.key, true),
            AccountMeta::new(*whirlpool.key, false),
            AccountMeta::new(*creator_ata.key, false),
            AccountMeta::new(*token_vault_a.key, false),
            AccountMeta::new(*temp_token_account.key, false), //token_owner_account_b, false),
            AccountMeta::new(*token_vault_b.key, false),
            AccountMeta::new(*tick_array_0.key, false),
            AccountMeta::new(*tick_array_1.key, false),
            AccountMeta::new(*tick_array_2.key, false),
            AccountMeta::new_readonly(*oracle.key, false),
        ],
    );

    invoke_signed(
        &swap_ix,
        &[
            spl_token_program.clone(),
            paycheck_account.clone(),
            whirlpool.clone(),
            creator_ata.clone(),
            token_vault_a.clone(),
            temp_token_account.clone(),
            token_vault_b.clone(),
            tick_array_0.clone(),
            tick_array_1.clone(),
            tick_array_2.clone(),
            oracle.clone(),
        ],
        &[&[
            PAYCHECK_SEED,
            &whirlpool.key.to_bytes(),
            &args.creator.to_bytes(),
            &[paycheck_data.bump],
        ]],
    )?;

    // Send the output to the receiver and executor

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

    let data = borsh::to_vec(&PaycheckInstructions::ExecutePaycheck(
        ExecutePaycheckArgs { creator },
    ))
    .map_err(|_| PaycheckProgramError::InvalidInstructionData)?;
    let paycheck_instruction_des = PaycheckInstructions::try_from_slice(&data);
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
