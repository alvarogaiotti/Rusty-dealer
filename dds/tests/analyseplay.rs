use dds::{
    DDSPlayAnalyzer, PlayAnalyzer, PlayTraceBin, PlayTracesBin, RankSeq, RawDDSRef, SuitSeq,
};
mod setup;
use setup::*;

const TRIES: usize = 200;

#[test]
fn analyse_play_test() {
    let deal = initialize_test();
    let contract = ContractMock {};
    let suitseq = SuitSeq::try_from([0i32, 0i32, 0i32, 0i32]).unwrap();
    let rankseq = RankSeq::try_from([4i32, 3i32, 12i32, 2i32]).unwrap();
    let play = PlayTraceBin::new(suitseq, rankseq);
    let analyzer = DDSPlayAnalyzer::new();
    let solvedplay = analyzer.analyze_play(&deal, &contract, play).unwrap();
    assert_eq!([2, 2, 2, 2, 2], solvedplay.tricks[..5]);
}

#[test]
fn analyse_all_play_test() {
    let mut deals_owner = Vec::with_capacity(TRIES);
    deals_owner.resize_with(TRIES, initialize_test);
    let deals = deals_owner.iter().collect();
    let suitseq = SuitSeq::try_from([0, 0, 0, 0]).unwrap();
    let rankseq = RankSeq::try_from([4, 3, 12, 2]).unwrap();
    let mut suitseqs = Vec::with_capacity(TRIES);
    let mut rankseqs = Vec::with_capacity(TRIES);
    suitseqs.resize_with(TRIES, || suitseq.clone());
    rankseqs.resize_with(TRIES, || rankseq.clone());
    let contracts_owner = Vec::from([ContractMock {}; TRIES]);
    let contracts = contracts_owner.iter().collect();
    let mut plays = PlayTracesBin::from_sequences(suitseqs, rankseqs).unwrap();
    let analyzer = DDSPlayAnalyzer::new();
    let solved_plays = analyzer
        .analyze_all_plays(deals, contracts, &mut plays)
        .unwrap();
    assert_eq!(TRIES, solved_plays.noOfBoards.try_into().unwrap());
    for plays in solved_plays.solved {
        assert_eq!([2, 2, 2, 2, 2], plays.tricks[..5]);
    }
}
