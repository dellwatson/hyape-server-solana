use anchor_lang::prelude::*;
use ephemeral_rollups_sdk::anchor::{commit, delegate, ephemeral};
use ephemeral_rollups_sdk::cpi::DelegateConfig;
use ephemeral_rollups_sdk::ephem::{commit_accounts, commit_and_undelegate_accounts};

declare_id!("FBxbhg65mbUwVwvvLR5Z4UC7vx45mhaHCy3Nr7LpjBvs");

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
    pub player: Pubkey,          // Player's address
    pub game_mode: String,            // 
    pub room_id: String,         // Room ID as string (e.g. "4aa2a8047921aa6a8e218b3ae69f57512b3f7c18")
    pub position: Position,      // Player's 3D position
    pub rotation: Rotation,      // Player's rotation
    pub animation: String,        // Simple animation ID
    pub model: String,            // Simple model ID  
    pub name: String,            // 
    pub color: String,            // 
    pub is_ready: String,            // 
    pub is_host: String,            // 
    pub is_finished: String,            // 
    // message ?
    // winCondition ? -> move winCondition to the server management instead
    //--------
    //score
    //avatar
}

#[ephemeral]
#[program]
pub mod game_server {
    use super::*;

    // Initialize the program, only do it ONCE forever, not in the game loop
    // to upload pda state into rollup ??
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let player_state = &mut ctx.accounts.player_state;
        
        // // Initialize with default values
        // player_state.position = Position::default();
        // player_state.rotation = Rotation::default();
        // player_state.animation = String::new();
        // player_state.model = String::new();
        // player_state.room_id = String::new();
        // player_state.game_mode = String::new();
        // player_state.name = String::new();
        // player_state.color = String::new();
        // player_state.is_ready = String::new();
        // player_state.is_host = String::new();
        // player_state.is_finished = String::new();
        // player_state.player = ctx.accounts.user.key();
        
        msg!("Program state initialized");
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
        player: Pubkey,
        room_id: String,
        game_mode: String, //
        animation: String,
        model: String,
        position: Position,
        rotation: Rotation,
        name: String,
        color: String,
        is_ready: String,
        is_host: String,
        is_finished: String,
        // win_condition: <Position, GameMode>,
    ) -> Result<()> {
        let player_state = &mut ctx.accounts.player_state;
        
        // Simple update without validation - just like anchor_counter
        player_state.player = player;
        player_state.position = position;
        player_state.rotation = rotation;
        player_state.animation = animation;
        player_state.model = model;
        player_state.room_id = room_id.clone();
        player_state.game_mode = game_mode;
        player_state.name = name;
        player_state.color = color;
        player_state.is_ready = is_ready;
        player_state.is_host = is_host;
        player_state.is_finished = is_finished;
        
        msg!("Updated player state for room {}", room_id);
        Ok(())

        // state -> win_condition => 
        // if player in exact position then auto execute set_winner for the room_id and store it
    }

    // Add this new instruction to your existing game_server module
    // This only commits the state and emits a winner event
    pub fn set_winner(ctx: Context<SetWinner>, room_id: String, winner: Pubkey) -> Result<()> {
        
        // First commit the state to the base layer
        commit_accounts(
            &ctx.accounts.payer,
            vec![&ctx.accounts.player_state.to_account_info()],
            &ctx.accounts.magic_context,
            &ctx.accounts.magic_program,
        )?;
        
        // Emit an event that the prize pool program can listen for
        emit!(WinnerDeclared {
            room_id: room_id.clone(),
            winner: winner,
            timestamp: Clock::get()?.unix_timestamp,
        });
        
        msg!("Winner set to: {} for room: {}", winner, room_id);
        Ok(())
    }

 
    pub fn set_winner_2(ctx: Context<CommitOrUndelegateState>, room_id: String, winner: Pubkey) -> Result<()> {
        // First commit the state to the base layer
        commit_accounts(
            &ctx.accounts.payer,
            vec![&ctx.accounts.player_state.to_account_info()],
            &ctx.accounts.magic_context,
            &ctx.accounts.magic_program,
        )?;
        
        // Emit an event that the prize pool program can listen for
        emit!(WinnerDeclared {
            room_id: room_id.clone(),
            winner: winner,
            timestamp: Clock::get()?.unix_timestamp,
        });
        
        msg!("Winner set to: {} for room: {}", winner, room_id);
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

    // finish the game and release the escrow to the winner
}


// Add this event that will be emitted when a winner is declared
#[event]
pub struct WinnerDeclared {
    pub room_id: String,
    pub winner: Pubkey,
    pub timestamp: i64,
}


//------------------------------

// Account validation structures
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = user,
        space = 8 + 32 + (4 + 50) + (4 + 50) + 12 + 16 + (4 + 50) + (4 + 50) + (4 + 50) + (4 + 50) + (4 + 50) + (4 + 50) + (4 + 50), // Account discriminator + PlayerState fields
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
    // #[account(mut, del, seeds = [PLAYER_STATE_SEED], bump)]
    #[account(mut, del)]
    pub pda: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct UpdatePlayerState<'info> {
    #[account(mut, seeds = [PLAYER_STATE_SEED], bump)]
    pub player_state: Account<'info, PlayerState>
    // pub player: Signer<'info>, // removing this // it should be the temp-keypair next time? or not
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

// Add this simple account validation struct
#[derive(Accounts)]
pub struct SetWinner<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    
    #[account(mut, seeds = [PLAYER_STATE_SEED], bump)]
    pub player_state: Account<'info, PlayerState>,
    
    /// CHECK: Magic context for Ephemeral Rollups
    pub magic_context: UncheckedAccount<'info>,
    
    /// CHECK: Magic program for Ephemeral Rollups
    pub magic_program: UncheckedAccount<'info>,
}

