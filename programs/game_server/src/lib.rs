use anchor_lang::prelude::*;
use ephemeral_rollups_sdk::anchor::{commit, delegate, ephemeral};
use ephemeral_rollups_sdk::cpi::DelegateConfig;
use ephemeral_rollups_sdk::ephem::{commit_accounts, commit_and_undelegate_accounts};

declare_id!("ACSCmLSGgn3kxQyqvDDeDqopzooM17aNEBuWt1pwHBXp");

// Constant seed for the player state PDA, similar to anchor_counter's TEST_PDA_SEED
pub const PLAYER_STATE_SEED: &[u8] = b"player-state";

// Position structure for 3D coordinates
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

// Rotation using quaternions
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default)]
pub struct Rotation {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

// Player state - direct equivalent to Counter in anchor_counter
#[account]
pub struct PlayerState {
    pub position: Position,      // Player's 3D position
    pub rotation: Rotation,      // Player's rotation
    pub animation_id: u8,        // Simple animation ID
    pub model_id: u8,            // Simple model ID  
    pub room_id: u8,             // Room ID
    pub player: Pubkey,          // Player's address
}

#[ephemeral]
#[program]
pub mod game_server {
    use super::*;

    // Initialize the player state (like anchor_counter's initialize)
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let player_state = &mut ctx.accounts.player_state;
        
        // Initialize with default values
        player_state.position = Position::default();
        player_state.rotation = Rotation::default();
        player_state.animation_id = 0;
        player_state.model_id = 0;
        player_state.room_id = 0;
        player_state.player = ctx.accounts.user.key();
        
        msg!("Player state initialized");
        Ok(())
    }

    // Delegate the player state to ER (identical to anchor_counter's delegate)
    pub fn delegate(ctx: Context<DelegateInput>) -> Result<()> {
        ctx.accounts.delegate_pda(
            &ctx.accounts.payer,
            &[PLAYER_STATE_SEED],
            DelegateConfig::default(),
        )?;
        
        msg!("Player state delegated to Ephemeral Rollup");
        Ok(())
    }

    // Update player state (similar to anchor_counter's increment)
    pub fn update_player(
        ctx: Context<UpdatePlayerState>, 
        room_id: u8,
        animation_id: u8,
        model_id: u8,
        position: Position,
        rotation: Rotation,
    ) -> Result<()> {
        let player_state = &mut ctx.accounts.player_state;
        
        // Simple update without validation - just like anchor_counter
        player_state.position = position;
        player_state.rotation = rotation;
        player_state.animation_id = animation_id;
        player_state.model_id = model_id;
        player_state.room_id = room_id;
        
        msg!("Updated player state for room {}", room_id);
        Ok(())
    }

    // Commit player state to base layer
    pub fn commit(ctx: Context<CommitOrUndelegateState>) -> Result<()> {
        commit_accounts(
            &ctx.accounts.payer,
            vec![&ctx.accounts.player_state.to_account_info()],
            &ctx.accounts.magic_context,
            &ctx.accounts.magic_program,
        )?;
        
        msg!("Player state committed to base layer");
        Ok(())
    }

    // Undelegate player state
    pub fn undelegate(ctx: Context<CommitOrUndelegateState>) -> Result<()> {
        commit_and_undelegate_accounts(
            &ctx.accounts.payer,
            vec![&ctx.accounts.player_state.to_account_info()],
            &ctx.accounts.magic_context,
            &ctx.accounts.magic_program,
        )?;
        
        msg!("Player state undelegated");
        Ok(())
    }
}

// Account validation structures
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = user,
        space = 8 + 12 + 16 + 1 + 1 + 1 + 32, // Account discriminator + PlayerState fields
        seeds = [PLAYER_STATE_SEED],
        bump
    )]
    pub player_state: Account<'info, PlayerState>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[delegate]
#[derive(Accounts)]
pub struct DelegateInput<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    
    /// CHECK: This account will be delegated to the ephemeral rollup
    #[account(mut, del, seeds = [PLAYER_STATE_SEED], bump)]
    pub pda: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct UpdatePlayerState<'info> {
    #[account(mut, seeds = [PLAYER_STATE_SEED], bump)]
    pub player_state: Account<'info, PlayerState>,
    
    pub player: Signer<'info>,
}

// Combined struct for both commit and undelegate to avoid the MagicProgram redefinition error
#[commit]
#[derive(Accounts)]
pub struct CommitOrUndelegateState<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    
    #[account(mut, seeds = [PLAYER_STATE_SEED], bump)]
    pub player_state: Account<'info, PlayerState>,
}