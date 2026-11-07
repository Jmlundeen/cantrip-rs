use cantrip_api::*;
use cantrip_mechanics::{assimilate, statics, targets, triggers};
use cantrip_oracle as oracle;

/// "Target creature card from an opponent's graveyard" (CR 115.1, 404.1): a creature card in the graveyard of a
/// player other than the ability's controller, still that card (CR 400.7).
fn creature_card_in_an_opponents_graveyard(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    targets::card_in_an_opponents_graveyard(q, asker, t)
        && matches!(t, Target::Object(id) if q.chars(id).is(CardType::Creature))
}

/// "When Borg Queen enters, assimilate target creature card from an opponent's graveyard." Assimilate moves that
/// card onto the battlefield under this ability's controller with a +1/+1 counter, as a Borg artifact creature
/// (CR 122.6, 205.1b, 614.12).
fn assimilate_target(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(card) = targets::object(g, item, 0) else { return Ok(()) };
    assimilate::assimilate(g, item, card)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::THIS_ENTERS,
        assimilate_target,
        "When Borg Queen enters, assimilate target creature card from an opponent's graveyard.",
    )
    .targets(&[target(creature_card_in_an_opponents_graveyard)])],
    statics: &[static_ability(
        Layer::PtModify,
        statics::your_artifact_creatures,
        statics::plus_pt::<2, 0>,
        "Artifact creatures you control get +2/+0.",
    )],
    ..oracle::BORG_QUEEN_PERFECTION_MANIFEST
};
