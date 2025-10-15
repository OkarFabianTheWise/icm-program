use crate::state::CreatorProfile;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct CreateProfile<'info> {
    #[account(
        init,
        payer = creator,
        // use the derive init space 
        space = 8 + CreatorProfile::INIT_SPACE,
        // space = 8 + 32 + 4 + 4 + 8 + 4 + 8,
        seeds = [b"creator_profile", creator.key().as_ref()],
        bump
    )]
    pub creator_profile: Account<'info, CreatorProfile>,
    #[account(mut)]
    pub creator: Signer<'info>,
    pub system_program: Program<'info, System>,
}

///### @check who has authority to create a profile
/// users are the ones to create a profile and each user can only create one profile per address
pub fn create_profile_handler(ctx: Context<CreateProfile>) -> Result<()> {
    let profile = &mut ctx.accounts.creator_profile;
    msg!("creating profile");
    
    profile.creator = ctx.accounts.creator.key();
    profile.pools_created = 0;
    profile.successful_pools = 0;
    profile.total_volume_managed = 0;
    profile.reputation_score = 0;
    profile.created_at = Clock::get()?.unix_timestamp;
    msg!("profile created");
    Ok(())
}
