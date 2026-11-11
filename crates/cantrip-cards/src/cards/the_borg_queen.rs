use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS_OR_ATTACKS;
use cantrip_mechanics::{assimilate, library, statics, targets};
use cantrip_oracle as oracle;

/// "Whenever The Borg Queen enters or attacks, target opponent exiles cards from the top of their library until they
/// exile a creature card. Assimilate that card. They put the rest on the bottom in a random order." (CR 701.13a,
/// 608.2d, 401.4, 614.12): the creature card this resolution's exiling stops on is put onto the battlefield under this
/// ability's controller's control as a Borg artifact creature with a +1/+1 counter, and the cards exiled before it go
/// to the bottom of their owner's library in a random order. A creature card can be a land (Dryad Arbor), so the walk
/// stops on any creature card, land or not; a library holding none is exiled whole, nothing is assimilated, and every
/// card of it goes to the bottom.
fn exile_until_creature_and_assimilate(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(opponent) = targets::player(g, item, 0) else { return Ok(()) };
    let (exiled, found) =
        library::exile_until(g, opponent, None, &|q, card| q.chars(card).is(CardType::Creature));
    if let Some(card) = found {
        assimilate::assimilate(g, item, card)?;
    }
    library::rest_of_exiled_to_bottom_in_random_order(g, &exiled);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    statics: &[static_ability(
        Layer::PtModify,
        statics::your_artifact_creatures,
        statics::plus_pt::<2, 2>,
        "Artifact creatures you control get +2/+2.",
    )],
    abilities: &[triggered(
        THIS_ENTERS_OR_ATTACKS,
        exile_until_creature_and_assimilate,
        "Whenever The Borg Queen enters or attacks, target opponent exiles cards from the top of their library until they exile a creature card. Assimilate that card. They put the rest on the bottom in a random order.",
    )
    .targets(&[targets::OPPONENT])],
    ..oracle::THE_BORG_QUEEN
};
