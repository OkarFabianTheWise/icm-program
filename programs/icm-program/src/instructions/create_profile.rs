use anchor_lang::prelude::*;
use crate::state::CreatorProfile;

#[derive(Accounts)]
pub struct CreateProfile<'info> {
    #[account(
        init,
        payer = creator,
        space = 8 + 32 + 4 + 4 + 8 + 4 + 8,
        seeds = [b"creator_profile", creator.key().as_ref()],
        bump
    )]
    pub creator_profile: Account<'info, CreatorProfile>,
    #[account(mut)]
    pub creator: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn create_profile_handler(
    ctx: Context<CreateProfile>,
) -> Result<()> {
    let profile = &mut ctx.accounts.creator_profile;
    
    profile.creator = ctx.accounts.creator.key();
    profile.pools_created = 0;
    profile.successful_pools = 0;
    profile.total_volume_managed = 0;
    profile.reputation_score = 0;
    profile.created_at = Clock::get()?.unix_timestamp;
    
    Ok(())
}
