use anchor_lang::prelude::borsh::BorshDeserialize;
use anchor_lang::prelude::*;
use anchor_lang::solana_program::sysvar::instructions as ix_sysvar;
use anchor_lang::system_program;
use solana_ed25519_program::{Ed25519SignatureOffsets, PUBKEY_SERIALIZED_SIZE};
use solana_sdk_ids::ed25519_program;

use crate::errors::ErrorCode;

#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub struct SemanticVersion {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

impl SemanticVersion {
    pub fn new(major: u16, minor: u16, patch: u16) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }
}

impl PartialOrd for SemanticVersion {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SemanticVersion {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.major.cmp(&other.major) {
            std::cmp::Ordering::Equal => match self.minor.cmp(&other.minor) {
                std::cmp::Ordering::Equal => self.patch.cmp(&other.patch),
                ord => ord,
            },
            ord => ord,
        }
    }
}

pub fn init_dynamic_pda<'info>(
    payer: &Signer<'info>,
    target_account: &AccountInfo<'info>,
    seeds: &[&[u8]],
    space: usize,
    owner: &Pubkey,
    system_program: &Program<'info, System>,
) -> Result<u8> {
    let (pda, bump) = Pubkey::find_program_address(seeds, &crate::ID);
    require_keys_eq!(target_account.key(), pda, ErrorCode::InvalidPDAAccount);

    if target_account.lamports() > 0 && !target_account.data_is_empty() {
        return Err(ErrorCode::AccountAlreadyInitialized.into());
    }

    let rent = Rent::get()?;
    let required_lamports = rent.minimum_balance(space);

    let bump_seed = &[bump];
    let mut signer_seeds = seeds.to_vec();
    signer_seeds.push(bump_seed);
    let signer_seeds = &[&signer_seeds[..]];

    let cpi_accounts = system_program::CreateAccount {
        from: payer.to_account_info(),
        to: target_account.clone(),
    };
    let cpi_context =
        CpiContext::new_with_signer(system_program.to_account_info(), cpi_accounts, signer_seeds);

    system_program::create_account(cpi_context, required_lamports, space as u64, owner)?;

    Ok(bump)
}

pub fn verify_tee_signature<T: BorshDeserialize>(
    instruction_sysvar: &AccountInfo,
    expected_tee_pubkey: &Pubkey,
) -> Result<T> {
    let ix_sysvar_account = instruction_sysvar.to_account_info();
    let current_ix_index = ix_sysvar::load_current_index_checked(&ix_sysvar_account)
        .map_err(|_| error!(ErrorCode::InvalidInstructionSysvar))?;

    require!(current_ix_index > 0, ErrorCode::InvalidInstructionSysvar);

    let ed_ix =
        ix_sysvar::load_instruction_at_checked((current_ix_index - 1) as usize, &ix_sysvar_account)
            .map_err(|_| error!(ErrorCode::InvalidInstructionSysvar))?;

    require!(
        ed_ix.program_id.as_ref() == ed25519_program::ID.as_ref(),
        ErrorCode::BadEd25519Program
    );
    require!(ed_ix.accounts.is_empty(), ErrorCode::BadEd25519Accounts);

    let ed_data = &ed_ix.data;
    require!(ed_data.len() >= 16, ErrorCode::InvalidInstructionSysvar);

    let offsets: Ed25519SignatureOffsets = bytemuck::try_pod_read_unaligned(&ed_data[2..16])
        .map_err(|_| error!(ErrorCode::InvalidInstructionSysvar))?;

    let pubkey_offset = offsets.public_key_offset as usize;
    let msg_offset = offsets.message_data_offset as usize;
    let msg_len = offsets.message_data_size as usize;

    let validator_pubkey_slice = &ed_data[pubkey_offset..(pubkey_offset + PUBKEY_SERIALIZED_SIZE)];
    let msg_bytes = &mut &ed_data[msg_offset..(msg_offset + msg_len)];

    require!(
        validator_pubkey_slice == expected_tee_pubkey.as_ref(),
        ErrorCode::InvalidValidatorTeeSigningPubkey
    );

    let message = T::deserialize(msg_bytes)?;
    Ok(message)
}

pub fn check_validation_threshold(
    current_validations: u32,
    required_validations: u32,
) -> Result<bool> {
    Ok(current_validations >= required_validations)
}

pub fn increment_validations(current: u32) -> Result<u32> {
    current.checked_add(1).ok_or(ErrorCode::Overflow.into())
}
