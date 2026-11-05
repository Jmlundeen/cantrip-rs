use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_mechanics::triggers::{PROWESS, THIS_ENTERS};
use cantrip_oracle as oracle;

const FOR_EACH_OPPONENT: TargetSpec =
    targets::UP_TO_ONE_NONLAND_PERMANENT_OF_EACH_OPPONENT.labelled("Nonland permanent {player} controls to put third from the top of its owner's library");

/// "... put up to one target nonland permanent that player controls into its owner's library third from the top": the targets
/// still legal when it resolves go together (CR 608.2b, 701.24); a token ceases to exist.
fn put_third_from_the_top(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let objects = targets::legal_objects(g, item);
    g.put_into_library(&objects, LibraryPlace::FromTop(2));
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        PROWESS,
        triggered(THIS_ENTERS, put_third_from_the_top, "When this creature enters, for each opponent, put up to one target nonland permanent that player controls into its owner's library third from the top.")
            .targets(&[FOR_EACH_OPPONENT]),
    ],
    ..oracle::RIPTIDE_GEARHULK
};
