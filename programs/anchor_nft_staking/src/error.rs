use anchor_lang::prelude::*;

#[error_code]
pub enum StakeError {
    #[msg("maximum stake reached")]
    MaxStake,

    #[msg("Time has not elapsed for staking")]
    TimeElapsed,

    #[msg("freeze period is not over yet")]
    FreezePeriod
}
