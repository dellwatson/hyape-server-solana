use anchor_lang::prelude::*;
use anchor_lang::solana_program::{program::invoke, system_instruction};

declare_id!("PRIZE_POOL_PROGRAM_ID_PLACEHOLDER");

// Constants
pub const PRIZE_ESCROW_SEED: &[u8] = b"prize-escrow";
pub const FEE_PERCENTAGE: u8 = 10; // 10% fee

#[program]
pub mod prize_pool {
    use super::*;

    // Deposit funds to the prize pool for a specific room
    // would also receive signature from game server
    pub fn UNSAFE_deposit(ctx: Context<Deposit>, room_id: String, amount: u64) -> Result<()> {
        let prize_pool = &mut ctx.accounts.prize_pool;
        let player = &ctx.accounts.player;
        
        // Initialize prize pool if it's new
        if prize_pool.total_lamports == 0 {
            prize_pool.room_id = room_id.clone();
            prize_pool.is_claimed = false;
            prize_pool.total_lamports = 0;
            prize_pool.player_count = 0;
        }
        
        // Verify that this is the correct room
        require!(prize_pool.room_id == room_id, ErrorCode::RoomIdMismatch);
        require!(!prize_pool.is_claimed, ErrorCode::PrizeAlreadyClaimed);
        
        // Transfer funds from player to prize escrow PDA
        invoke(
            &system_instruction::transfer(
                player.key,
                ctx.accounts.prize_escrow.key,
                amount,
            ),
            &[
                player.to_account_info(),
                ctx.accounts.prize_escrow.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;
        
        // Calculate fee
        let fee_amount = (amount * FEE_PERCENTAGE as u64) / 100;
        let prize_amount = amount - fee_amount;
        
        // Update prize pool state
        prize_pool.total_lamports += prize_amount;
        prize_pool.player_count += 1;
        
        msg!("Player deposited {} lamports to prize pool for room: {}", amount, room_id);
        Ok(())
    }

    // Claim prize - the winner directly claims here
    // In a real implementation, we would verify the winner with a signature or other proof
    // so instead this should have another params for signature with offchain (or perhaps delegate it into rollup too?)
    pub fn UNSAFE_claim_prize(ctx: Context<ClaimPrize>, room_id: String
    // add off-chain signature and decrypt it here to match with msg.sender (actual account executor)
    ) -> Result<()> {
        let prize_pool = &mut ctx.accounts.prize_pool;
        let claimer = &ctx.accounts.claimer;
        let prize_escrow = &ctx.accounts.prize_escrow;
        
        // Verify that this is the correct room
        require!(prize_pool.room_id == room_id, ErrorCode::RoomIdMismatch);
        
        // Verify prize hasn't been claimed yet
        require!(!prize_pool.is_claimed, ErrorCode::PrizeAlreadyClaimed);
        
        // IMPORTANT: In a production system, we would verify that the claimer
        // is actually the winner here. For simplicity, we're allowing anyone
        // to claim for now, but you would add verification logic:
        //
        // TODO: Add verification that claimer is the actual winner
        // This could be:
        // 1. A signature from the game server
        // 2. Reading the committed game state to verify winner
        // 3. An admin/authority approval
        
        // Get the prize amount
        let prize_amount = prize_pool.total_lamports;
        
        // Transfer funds from prize pool PDA to claimer
        **prize_escrow.try_borrow_mut_lamports()? -= prize_amount;
        **claimer.try_borrow_mut_lamports()? += prize_amount;
        
        // Mark as claimed
        prize_pool.is_claimed = true;
        
        msg!("Prize of {} lamports claimed from room: {}", prize_amount, room_id);
        Ok(())
    }

    // For admin use - we could add fee withdrawal or other admin functions
    // pub fn withdraw_fees(ctx: Context<WithdrawFees>) -> Result<()> {
    //     // Function to withdraw accumulated fees
    //     // Would be implemented in a production system
    //     Ok(())
    // }
}

// Account structures
#[account]
pub struct PrizePool {
    pub room_id: String,          // Room ID to match with game server
    pub total_lamports: u64,      // Total prize amount in lamports
    pub is_claimed: bool,         // Whether the prize has been claimed
    pub player_count: u8,         // Number of players who joined
}

// Context structures for instructions
#[derive(Accounts)]
#[instruction(room_id: String, amount: u64)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub player: Signer<'info>,
    
    #[account(
        init_if_needed,
        payer = player,
        space = 8 + 4 + 50 + 8 + 1 + 1, // Account discriminator + PrizePool fields
        seeds = [room_id.as_bytes()],
        bump
    )]
    pub prize_pool: Account<'info, PrizePool>,
    
    #[account(
        mut,
        seeds = [PRIZE_ESCROW_SEED, room_id.as_bytes()],
        bump,
    )]
    /// CHECK: This is a PDA that will hold funds
    pub prize_escrow: AccountInfo<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(room_id: String)]
pub struct ClaimPrize<'info> {
    #[account(mut)]
    pub claimer: Signer<'info>,
    
    #[account(
        mut,
        seeds = [room_id.as_bytes()],
        bump,
    )]
    pub prize_pool: Account<'info, PrizePool>,
    
    #[account(
        mut,
        seeds = [PRIZE_ESCROW_SEED, room_id.as_bytes()],
        bump,
    )]
    /// CHECK: This is a PDA that will hold funds
    pub prize_escrow: AccountInfo<'info>,
    
    pub system_program: Program<'info, System>,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Room ID mismatch")]
    RoomIdMismatch,
    #[msg("Prize has already been claimed")]
    PrizeAlreadyClaimed,
}