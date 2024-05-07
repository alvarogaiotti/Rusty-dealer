use crate::prelude::*;
use dds::*;
use itertools::*;
use std::cmp::Ordering;

pub struct TraceSolved {
    pub trace: PlaySequence,
    pub results: SolvedPlay,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrickPosition {
    First = 0,
    Second,
    Third,
    Last,
}

impl From<u8> for TrickPosition {
    fn from(value: u8) -> Self {
        match value % 4 {
            0 => Self::First,
            1 => Self::Second,
            2 => Self::Third,
            3 => Self::Last,
            _ => unreachable!(),
        }
    }
}
impl From<usize> for TrickPosition {
    fn from(value: usize) -> Self {
        match value % 4 {
            0 => Self::First,
            1 => Self::Second,
            2 => Self::Third,
            3 => Self::Last,
            _ => unreachable!(),
        }
    }
}

/// Analyse the player performace based on the DDS play analysis
pub fn analyse_player_performance(
    player: Seat,
    contract: Contract,
    play_result_trace: TraceSolved,
) {
    let TraceSolved { trace, results } = play_result_trace;
    let mut tricks_iterator = results.iter().peekable();
    let mut correct_played = 0;
    let mut total_played = 0;
    let mut leader = contract.leader();
    let mut last_result = tricks_iterator
        .next()
        .expect("there should always be a result calculated before the attack");
    if leader == player {
        correct_played += (last_result
            == *tricks_iterator
                .peek()
                .expect("there should always be a result calculated after the attack"))
            as usize;
    }
    let winners_table = winners(trace, contract);
    for (i, winner) in winners_table.iter().enumerate() {
        let my_position = player_position(leader, player);
        // The result sequence starts with the result before the first trick
        if results[my_position as usize * (i + 1) + 1] <= results[my_position as usize * (i + 1)] {
            correct_played += 1;
        }
        leader = Into::<Seat>::into(winner);
    }
}

fn player_position(leader: Seat, player: Seat) -> TrickPosition {
    (4 - leader as u8 + player as u8).into()
}

fn winners(play_result_trace: PlaySequence, contract: Contract) -> Vec<usize> {
    let strain = contract.strain();
    play_result_trace
        .into_iter()
        .chunks(4)
        .into_iter()
        .map(|chunk| {
            if strain == Strain::NoTrumps {
                max_no_trump(chunk.into_iter())
            } else {
                max_trump(
                    chunk.into_iter(),
                    strain.try_into().expect("just checked: it's not NoTrumps"),
                )
            }
        })
        .collect()
}

fn max_no_trump<I>(trick: I) -> usize
where
    I: Iterator<Item = Card>,
{
    trick
        .enumerate()
        .max_by(|(_, card1), (_, card2)| {
            if card1.suit() != card2.suit() {
                Ordering::Greater
            } else {
                card1.rank().cmp(&card2.rank())
            }
        })
        .expect("shouldn't be calling this function whithout a populated trick")
        .0
}

fn max_trump<I>(trick: I, trump: Suit) -> usize
where
    I: Iterator<Item = Card>,
{
    trick
        .enumerate()
        .max_by(|(_, card1), (_, card2)| {
            if card1.suit() != card2.suit() {
                if card2.suit() == trump {
                    Ordering::Greater
                } else {
                    Ordering::Less
                }
            } else {
                card1.rank().cmp(&card2.rank())
            }
        })
        .expect("shouldn't be calling this function whithout a populated trick")
        .0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::*;
    use dds::*;

    #[test]
    fn calculate_correct_positions() {
        let vec: Vec<Seat> = vec![1u8, 2, 3, 0, 1].into_iter().map(Into::into).collect();
        let player: Vec<Seat> = vec![0u8, 2, 4, 2, 8].into_iter().map(Into::into).collect();
        for (index, (start, player)) in vec.iter().zip(player.iter()).enumerate() {
            assert_eq!(
                TrickPosition::from(3usize + index),
                player_position(*start, *player)
            );
        }
    }
}
