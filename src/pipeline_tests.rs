use super::{Demand, Pipeline};

const RESOURCE: u32 = 7;

fn ready() -> Pipeline<u32> {
    let mut pipeline = Pipeline::Missing;
    pipeline.store(Some(RESOURCE));
    pipeline
}

fn failed() -> Pipeline<u32> {
    let mut pipeline = Pipeline::Missing;
    pipeline.store(None);
    pipeline
}

#[test]
fn a_missing_pipeline_asks_for_creation() {
    let pipeline: Pipeline<u32> = Pipeline::Missing;

    assert_eq!(pipeline.demand(|_| true), Demand::Create);
    assert!(Pipeline::<u32>::Missing.ready().is_none());
}

#[test]
fn a_failed_creation_is_never_retried() {
    let pipeline = failed();

    for _ in 0..1000 {
        assert_eq!(pipeline.demand(|_| true), Demand::Skip);
    }
    assert_eq!(pipeline.demand(|_| false), Demand::Skip);
}

#[test]
fn a_failed_pipeline_offers_no_resource() {
    assert!(failed().ready().is_none());
}

#[test]
fn a_ready_pipeline_is_reused_while_it_matches() {
    let pipeline = ready();

    assert_eq!(pipeline.demand(|_| true), Demand::Reuse);
}

#[test]
fn a_ready_pipeline_is_recreated_when_it_stops_matching() {
    let pipeline = ready();

    assert_eq!(pipeline.demand(|_| false), Demand::Create);
}

#[test]
fn a_ready_pipeline_hands_out_its_resource() {
    let mut pipeline = ready();

    assert_eq!(pipeline.ready().copied(), Some(RESOURCE));
}

#[test]
fn the_matching_test_only_runs_for_a_ready_pipeline() {
    let mut consulted = false;
    failed().demand(|_| {
        consulted = true;
        true
    });

    assert!(!consulted);
}

#[test]
fn resetting_allows_a_new_attempt_after_a_failure() {
    let mut pipeline = failed();

    pipeline.reset();

    assert_eq!(pipeline.demand(|_| true), Demand::Create);
}

#[test]
fn a_second_attempt_can_succeed_after_a_reset() {
    let mut pipeline = failed();
    pipeline.reset();

    pipeline.store(Some(RESOURCE));

    assert_eq!(pipeline.demand(|_| true), Demand::Reuse);
    assert_eq!(pipeline.ready().copied(), Some(RESOURCE));
}
