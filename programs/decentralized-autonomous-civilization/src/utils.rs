use anchor_lang::prelude::*;
use anchor_lang::system_program;

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
) -> Result<(u8)> {
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
    let cpi_context = CpiContext::new_with_signer(
        system_program.to_account_info(),
        cpi_accounts,
        signer_seeds,
    );

    system_program::create_account(
        cpi_context,
        required_lamports,
        space as u64,
        owner,
    )?;

    Ok((bump))
}