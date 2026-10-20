//! A two-card shuffle can leave the order unchanged. Check the empty-search scenario's random generator around
//! its search as well as its zone assertions, so omitting the shuffle cannot pass by producing the same order.

use cantrip_engine::{PlayerId, RequestKind};
use cantrip_script::{Advance, GameScript, Scripted};

#[test]
fn finding_no_basics_still_shuffles() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../scenarios/fblthp-knows-the-way-finds-nothing-without-basics.cantrip");
    let mut table = Scripted::deal(GameScript::load(&path).unwrap()).unwrap();
    let mut before_search = None;
    loop {
        match table.advance(&[PlayerId(0), PlayerId(1)]).unwrap() {
            Advance::Ask(expected) => {
                if matches!(expected.request.kind, RequestKind::SearchLibrary { .. }) {
                    assert!(before_search.is_none(), "the scenario searches only once");
                    before_search = Some(table.game().state.rng.state);
                }
                table.answered(expected.intent).unwrap();
            }
            Advance::Check(_) => {}
            Advance::Done => break,
        }
    }
    table.verdict().unwrap();
    assert_ne!(
        before_search.expect("the empty search must be asked"),
        table.game().state.rng.state,
        "the library must be shuffled even when no cards are found"
    );
}
