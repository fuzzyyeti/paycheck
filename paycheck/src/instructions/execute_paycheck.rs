use crate::consts::{MEMO_PROGRAM_ID, PAYCHECK_SEED, SWAP_DISCRIMINATOR};
use crate::error::PaycheckProgramError;
use crate::processor::PaycheckInstructions;
use crate::state::Paycheck;
use crate::ID;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::account_info::{next_account_info, AccountInfo};
use solana_program::clock::Clock;
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::msg;
use solana_program::program::{invoke, invoke_signed};
use solana_program::program_error::ProgramError;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use solana_program::rent::Rent;
use solana_program::sysvar::Sysvar;
use spl_associated_token_account::get_associated_token_address;
use spl_token::state::Account;
use whirlpools_state::{SwapArgs, Whirlpool};

#[derive(Debug, BorshDeserialize, BorshSerialize)]
pub struct ExecutePaycheckArgs {
    pub creator: Pubkey,
    pub a_to_b: bool,
}
pub fn process_execute_paycheck(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: ExecutePaycheckArgs,
) -> Result<(), ProgramError> {
    let account_info_iter = &mut accounts.iter();
    let payer = next_account_info(account_info_iter)?;
    let paycheck = next_account_info(account_info_iter)?;
    let whirlpool = next_account_info(account_info_iter)?;
    let treasury_mint = next_account_info(account_info_iter)?;
    let treasury_token_account = next_account_info(account_info_iter)?;
    let temp_mint = next_account_info(account_info_iter)?;
    let temp_token_account = next_account_info(account_info_iter)?;
    let receiver_token_account = next_account_info(account_info_iter)?;
    let payer_token_account = next_account_info(account_info_iter)?;
    let token_vault_a = next_account_info(account_info_iter)?;
    let token_vault_b = next_account_info(account_info_iter)?;
    let tick_array_0 = next_account_info(account_info_iter)?;
    let tick_array_1 = next_account_info(account_info_iter)?;
    let tick_array_2 = next_account_info(account_info_iter)?;
    let oracle = next_account_info(account_info_iter)?;
    let spl_token_program = next_account_info(account_info_iter)?;
    let memo_program = next_account_info(account_info_iter)?;
    let whirlpool_data = Whirlpool::try_from_slice(&whirlpool.data.borrow())?;
    let paycheck_data: Paycheck = Paycheck::try_from_slice(&paycheck.data.borrow())?;
    let required_lamports = Rent::get()?.minimum_balance(Account::LEN);

    // Mints come from the whirlpool make sure the input and output are correct
    assert_eq!(paycheck_data.a_to_b, args.a_to_b);

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
        &temp_mint.key,
        &paycheck.key,
    )?;

    invoke_signed(
        &create_account_ix,
        &[temp_token_account.clone(), temp_mint.clone()],
        &[&[
            PAYCHECK_SEED,
            &whirlpool.key.to_bytes(),
            &args.creator.to_bytes(),
            &[paycheck_data.bump],
        ]],
    )?;

    // Perform the swap
    let amount = paycheck_data.tip + paycheck_data.amount;
    let input_args = SwapArgs {
        swap_discriminator: SWAP_DISCRIMINATOR,
        amount,
        other_amount_threshold: u64::MAX,
        sqrt_price_limit: 0,
        amount_specified_is_input: false,
        a_to_b: args.a_to_b,
        remaining_accounts_info: None,
    };

    let (
        mint_a,
        mint_b,
        token_account_a,
        token_account_b) = if args.a_to_b {
        (treasury_mint, temp_mint, treasury_token_account, temp_token_account)
    } else {
        (temp_mint, treasury_mint, temp_token_account, treasury_token_account)
    };

    let swap_ix = Instruction::new_with_borsh(
        whirlpools_state::ID,
        &input_args,
        vec![
            AccountMeta::new_readonly(*spl_token_program.key, false),
            AccountMeta::new_readonly(*spl_token_program.key, false),
            AccountMeta::new_readonly(*memo_program.key, false),
            AccountMeta::new(*paycheck.key, true),
            AccountMeta::new(*whirlpool.key, false),
            AccountMeta::new_readonly(*mint_a.key, false),
            AccountMeta::new_readonly(*mint_b.key, false),
            AccountMeta::new(*token_account_a.key, false),
            AccountMeta::new(*token_vault_a.key, false),
            AccountMeta::new(*token_account_b.key, false),
            AccountMeta::new(*token_vault_b.key, false),
            AccountMeta::new(*tick_array_0.key, false),
            AccountMeta::new(*tick_array_1.key, false),
            AccountMeta::new(*tick_array_2.key, false),
            AccountMeta::new(*oracle.key, false),
        ],
    );

    invoke_signed(
        &swap_ix,
        &[
            spl_token_program.clone(),
            spl_token_program.clone(),
            memo_program.clone(),
            paycheck.clone(),
            whirlpool.clone(),
            mint_a.clone(),
            mint_b.clone(),
            token_account_a.clone(),
            token_vault_a.clone(),
            token_account_b.clone(),
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

    msg!("Temp owner: {:?}", temp_token_account.owner);
    msg!("Receiver owner: {:?}", receiver_token_account.owner);
    msg!("Receiver: {:?}", receiver_token_account.key);
    // Send the output to the receiver and executor
    let transfer_ix = spl_token::instruction::transfer(
        &spl_token::id(),
        temp_token_account.key,
        receiver_token_account.key,
        &paycheck.key,
        &[],
        paycheck_data.amount,
    )?;

    invoke_signed(
        &transfer_ix,
        &[
            temp_token_account.clone(),
            receiver_token_account.clone(),
            paycheck.clone(),
        ],
        &[&[
            PAYCHECK_SEED,
            &whirlpool.key.to_bytes(),
            &args.creator.to_bytes(),
            &[paycheck_data.bump],
        ]],
    )?;

    // // Close the temp token account
    // let close_account_ix = spl_token::instruction::close_account(
    //     &spl_token::id(),
    //     temp_token_account.key,
    //     payer_token_account.key,
    //     &payer.key,
    //     &[&payer.key],
    // )?;
    //
    // invoke_signed(
    //     &close_account_ix,
    //     &[
    //         temp_token_account.clone(),
    //         payer_token_account.clone(),
    //         payer.clone(),
    //     ],
    //     &[&[
    //         PAYCHECK_SEED,
    //         &whirlpool.key.to_bytes(),
    //         &args.creator.to_bytes(),
    //         &[paycheck_data.bump],
    //     ]],
    // )?;
    // Update the paycheck account
    let current_timestamp = Clock::get()?.unix_timestamp;
    msg!("Current timestamp: {}", current_timestamp);

    Ok(())
}

pub fn execute_paycheck_ix(
    payer: Pubkey,
    creator: Pubkey,
    whirlpool: Pubkey,
    treasury_mint: Pubkey,
    temp_mint: Pubkey,
    treasury_token_account: Pubkey,
    temp_token_account: Pubkey,
    token_vault_a: Pubkey,
    token_vault_b: Pubkey,
    tick_array_0: Pubkey,
    tick_array_1: Pubkey,
    tick_array_2: Pubkey,
    oracle: Pubkey,
    a_to_b: bool,
) -> Result<Instruction, PaycheckProgramError> {
    let paycheck = Pubkey::find_program_address(
        &[PAYCHECK_SEED, &whirlpool.to_bytes(), &creator.to_bytes()],
        &ID,
    )
    .0;

    let data = borsh::to_vec(&PaycheckInstructions::ExecutePaycheck(
        ExecutePaycheckArgs { creator, a_to_b },
    ))
    .map_err(|_| PaycheckProgramError::InvalidInstructionData)?;

    let receiver_token_account = get_associated_token_address(&creator, &temp_mint);
    println!("Receiver token account: {:?}", receiver_token_account);
    println!("Receiver token account owner: {:?}", creator);
    println!("Receiver mint: {:?}", temp_mint);
    let payer_token_account = get_associated_token_address(&payer, &temp_mint);

    Ok(Instruction {
        program_id: ID,
        accounts: vec![
            AccountMeta::new(payer, true),
            AccountMeta::new(paycheck, false),
            AccountMeta::new(whirlpool, false),
            AccountMeta::new(treasury_mint, false),
            AccountMeta::new(treasury_token_account, false),
            AccountMeta::new(temp_mint, false),
            AccountMeta::new(temp_token_account, true),
            AccountMeta::new(receiver_token_account, false),
            AccountMeta::new(payer_token_account, false),
            AccountMeta::new(token_vault_a, false),
            AccountMeta::new(token_vault_b, false),
            AccountMeta::new(tick_array_0, false),
            AccountMeta::new(tick_array_1, false),
            AccountMeta::new(tick_array_2, false),
            AccountMeta::new(oracle, false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(MEMO_PROGRAM_ID, false),
            AccountMeta::new_readonly(whirlpools_state::ID, false),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
        ],
        data,
    })
}
