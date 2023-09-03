use super::{
    ddserror::{DDSError, DDSErrorKind},
    ddsffi::{boards, boardsPBN, deal, dealPBN},
    AsDDSContract, Mode, RawDDS, Solutions, Target, MAXNOOFBOARDSEXPORT,
};
use std::{ffi::c_int, fmt::Display};

// See https://stackoverflow.com/questions/28028854/how-do-i-match-enum-values-with-an-integer

#[derive(Debug, RawDDS, Default)]
pub struct DDSCurrTrickSuit(#[raw] [c_int; 3]);

#[derive(Debug, RawDDS, Default)]
pub struct DDSCurrTrickRank(#[raw] [c_int; 3]);

pub enum DDSSuitEncoding {
    Spades = 0,
    Hearts = 1,
    Diamonds = 2,
    Clubs = 3,
    NoTrump = 4,
}

macro_rules! impl_tryfrom_dds {
    ($from:ty, $to:ty, $err:ty) => {
        impl std::convert::TryFrom<$from> for $to {
            type Error = $err;

            fn try_from(v: $from) -> Result<Self, Self::Error> {
                match v {
                    0 => Ok(Self::Spades),
                    1 => Ok(Self::Hearts),
                    2 => Ok(Self::Diamonds),
                    3 => Ok(Self::Clubs),
                    4 => Ok(Self::NoTrump),
                    _ => Err(Self::Error::TrumpUnconvertable),
                }
            }
        }
    };
}
impl_tryfrom_dds!(u8, DDSSuitEncoding, DDSDealConstructionError);
impl_tryfrom_dds!(u16, DDSSuitEncoding, DDSDealConstructionError);
impl_tryfrom_dds!(u32, DDSSuitEncoding, DDSDealConstructionError);
impl_tryfrom_dds!(usize, DDSSuitEncoding, DDSDealConstructionError);
impl_tryfrom_dds!(i8, DDSSuitEncoding, DDSDealConstructionError);
impl_tryfrom_dds!(i16, DDSSuitEncoding, DDSDealConstructionError);
impl_tryfrom_dds!(i32, DDSSuitEncoding, DDSDealConstructionError);
impl_tryfrom_dds!(isize, DDSSuitEncoding, DDSDealConstructionError);

#[derive(Debug, Default)]
pub enum DDSHandEncoding {
    #[default]
    North = 0,
    East = 1,
    South = 2,
    West = 3,
}

macro_rules! impl_tryfrom_dds_hand {
    ($from:ty) => {
        impl std::convert::TryFrom<$from> for DDSHandEncoding {
            type Error = DDSDealConstructionError;

            fn try_from(v: $from) -> Result<Self, Self::Error> {
                match v {
                    0 => Ok(Self::North),
                    1 => Ok(Self::East),
                    2 => Ok(Self::South),
                    3 => Ok(Self::West),
                    _ => Err(Self::Error::TrumpUnconvertable),
                }
            }
        }
    };
}

impl_tryfrom_dds_hand!(u8);
impl_tryfrom_dds_hand!(u16);
impl_tryfrom_dds_hand!(u32);
impl_tryfrom_dds_hand!(usize);
impl_tryfrom_dds_hand!(i8);
impl_tryfrom_dds_hand!(i16);
impl_tryfrom_dds_hand!(i32);
impl_tryfrom_dds_hand!(isize);

#[derive(Debug, RawDDS)]
pub struct DDSDealRepr(#[raw] [[u32; 4]; 4]);

impl DDSDealRepr {
    pub fn new(data: [[u32; 4]; 4]) -> Self {
        Self(data)
    }

    pub fn as_slice(self) -> [[u32; 4]; 4] {
        self.0
    }
}

#[derive(Debug, RawDDS)]
pub struct DDSDealPBNRepr(#[raw] [std::ffi::c_char; 80]);

pub trait AsDDSDeal {
    fn as_dds_deal(&self) -> DDSDealRepr;
}

pub struct DDSDealBuilder {
    trump: Option<DDSSuitEncoding>,
    first: Option<DDSHandEncoding>,
    current_trick_suit: Option<DDSCurrTrickSuit>,
    current_trick_rank: Option<DDSCurrTrickRank>,
    remain_cards: Option<DDSDealRepr>,
}
#[derive(Debug)]
pub enum DDSDealConstructionError {
    DuplicatedCard(c_int, c_int),
    CurrentTrickRankNotSet,
    CurrentTrickSuitNotSet,
    DealNotLoaded,
    TrumpNotDeclared,
    FirstNotDeclared,
    FirstUnconvertable,
    TrumpUnconvertable,
}

impl Display for DDSDealConstructionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::CurrentTrickRankNotSet => write!(
                f,
                "current trick rank is not set while current trick suit is"
            ),
            Self::CurrentTrickSuitNotSet => write!(
                f,
                "current trick suit is not set while current trick rank is"
            ),
            Self::DuplicatedCard(suit, rank) => {
                let card = dds_card_tuple_to_string(suit, rank);
                write!(f, "duplicated card: {card}")
            }
            Self::DealNotLoaded => write!(f, "deal not loaded"),
            Self::FirstNotDeclared => write!(f, "leader not declared"),
            Self::TrumpNotDeclared => write!(f, "trump not declared"),
            Self::FirstUnconvertable => {
                write!(f, "first cannot be converted from the value you provided")
            }
            Self::TrumpUnconvertable => {
                write!(f, "first cannot be converted from the value you provided")
            }
        }
    }
}
impl std::error::Error for DDSDealConstructionError {}

impl DDSDealBuilder {
    pub fn new() -> Self {
        DDSDealBuilder {
            trump: None,
            first: None,
            current_trick_suit: None,
            current_trick_rank: None,
            remain_cards: None,
        }
    }
    pub fn trump(mut self, trump: DDSSuitEncoding) -> Self {
        self.trump = Some(trump);
        self
    }
    pub fn first(mut self, first: DDSHandEncoding) -> Self {
        self.first = Some(first);
        self
    }
    pub fn remain_cards(mut self, remain_cards: DDSDealRepr) -> Self {
        self.remain_cards = Some(remain_cards);
        self
    }
    pub fn current_trick_suit(mut self, current_trick_suit: DDSCurrTrickSuit) -> Self {
        self.current_trick_suit = Some(current_trick_suit);
        self
    }
    pub fn current_trick_rank(mut self, current_trick_rank: DDSCurrTrickRank) -> Self {
        self.current_trick_rank = Some(current_trick_rank);
        self
    }
    pub fn build(self) -> Result<DDSDeal, DDSDealConstructionError> {
        let remain_cards = self
            .remain_cards
            .ok_or(DDSDealConstructionError::DealNotLoaded)?;
        let trump = self
            .trump
            .ok_or(DDSDealConstructionError::TrumpNotDeclared)?;
        let first = self
            .first
            .ok_or(DDSDealConstructionError::FirstNotDeclared)?;
        let (current_trick_suit, current_trick_rank) =
            match (self.current_trick_suit, self.current_trick_rank) {
                (Some(suits), Some(ranks)) => Ok((suits, ranks)),
                (None, None) => Ok(Default::default()),
                (None, _) => Err(DDSDealConstructionError::CurrentTrickSuitNotSet),
                (_, None) => Err(DDSDealConstructionError::CurrentTrickRankNotSet),
            }?;
        Ok(DDSDeal::new(
            trump,
            first,
            current_trick_rank,
            current_trick_suit,
            remain_cards,
        ))
    }
}

#[derive(RawDDS, Debug)]
pub struct DDSDeal {
    #[raw]
    raw: deal,
}

impl DDSDeal {
    pub(super) fn new(
        trump: DDSSuitEncoding,
        first: DDSHandEncoding,
        current_trick_rank: DDSCurrTrickRank,
        current_trick_suit: DDSCurrTrickSuit,
        remain_cards: DDSDealRepr,
    ) -> Self {
        Self {
            raw: deal {
                trump: trump as c_int,
                first: first as c_int,
                currentTrickSuit: current_trick_suit.get_raw(),
                currentTrickRank: current_trick_rank.get_raw(),
                remainCards: remain_cards.get_raw(),
            },
        }
    }
}

#[derive(RawDDS, Debug)]
pub(super) struct DDSDealPBN {
    #[raw]
    raw: dealPBN,
}

impl DDSDealPBN {
    pub fn new(
        trump: c_int,
        first: c_int,
        current_trick_rank: DDSCurrTrickRank,
        current_trick_suit: DDSCurrTrickSuit,
        remain_cards: DDSDealPBNRepr,
    ) -> Self {
        Self {
            raw: dealPBN {
                trump,
                first,
                currentTrickSuit: current_trick_suit.get_raw(),
                currentTrickRank: current_trick_rank.get_raw(),
                remainCards: remain_cards.get_raw(),
            },
        }
    }
}

fn dds_card_tuple_to_string(suit: c_int, rank: c_int) -> String {
    let rankstr = match rank {
        0b_100 => "2",
        0b_1000 => "3",
        0b_10000 => "4",
        0b_100000 => "5",
        0b_1000000 => "6",
        0b_10000000 => "7",
        0b_100000000 => "8",
        0b_1000000000 => "9",
        0b_10000000000 => "10",
        0b_100000000000 => "J",
        0b_1000000000000 => "Q",
        0b_10000000000000 => "K",
        0b_100000000000000 => "A",
        _ => unreachable!("sanity checks on rank not performed, i'm panicking"),
    };
    let suitstr = match suit {
        0 => "♠",
        1 => "♥",
        2 => "◆",
        3 => "♣",
        _ => unreachable!("sanity checks on suit not performed, i'm panicking"),
    };
    let mut res = String::with_capacity(2);
    res.push_str(suitstr);
    res.push_str(rankstr);
    res
}

#[derive(RawDDS, Debug)]
pub struct Boards {
    #[raw]
    raw: boards,
}

impl boards {
    pub fn new<D: AsDDSDeal, C: AsDDSContract, const N: usize>(
        deals: &[&D; N],
        contracts: &[C; N],
        target: [Target; N],
        solution: [Solutions; N],
        mode: [Mode; N],
    ) -> Result<Self, DDSError> {
        if N > 200 {
            return Err(DDSError::from(DDSErrorKind::ChunkSize));
        }
        let length_check = N as i32;
        Ok(boards {
            noOfBoards: length_check,
            // SAFETY: Length if 200
            deals: boards::setup_deals(deals, contracts).try_into().unwrap(),
            target: boards::convert_sequence(target).try_into().unwrap(),
            solutions: boards::convert_sequence(solution).try_into().unwrap(),
            mode: boards::convert_sequence(mode).try_into().unwrap(),
        })
    }

    fn setup_deals<D: AsDDSDeal, C: AsDDSContract, const N: usize>(
        deals: &[&D; N],
        contracts: &[C; N],
    ) -> Vec<deal> {
        let complete_deals = deals.iter().zip(contracts.iter());
        let mut deals: Vec<deal> = Vec::with_capacity(MAXNOOFBOARDSEXPORT);
        deals.extend(complete_deals.map(|(d, c)| {
            let (trump, first) = c.as_dds_contract();
            deal {
                trump,
                first,
                currentTrickSuit: [0; 3],
                currentTrickRank: [0; 3],
                remainCards: d.as_dds_deal().as_slice(),
            }
        }));
        deals.resize_with(MAXNOOFBOARDSEXPORT, deal::default);
        deals
    }

    fn convert_sequence<T: Into<i32>, const N: usize>(sequence: [T; N]) -> Vec<i32> {
        let mut targets: Vec<i32> = Vec::with_capacity(MAXNOOFBOARDSEXPORT);
        targets.extend(sequence.map(|t| t.into()));
        targets.resize_with(MAXNOOFBOARDSEXPORT, Default::default);
        targets
    }
}
