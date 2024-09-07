use bytemuck::{Pod, Zeroable};
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::pubkey::Pubkey;
use crate::processor::PaycheckInstructions;

#[repr(C)]
#[derive(Debug, PartialEq, Clone, Copy, Zeroable, Pod)]
pub struct ConfigArgs {
    pub admin: Pubkey,
}

pub fn create_config_ix(
    program_id: &Pubkey,
    config_args: ConfigArgs
) -> Instruction {
    let mut data = bytemuck::bytes_of(&PaycheckInstructions::CreateConfig).to_vec();
    println!("data: {:?}", data);
    data.extend_from_slice(bytemuck::bytes_of(&config_args));
    println!("data: {:?}", data);
    Instruction {
        program_id: *program_id,
        accounts: vec![
            AccountMeta::new_readonly(config_args.admin, true),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),],
        data
    }
}