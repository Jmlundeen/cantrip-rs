use cantrip_api::*;
use cantrip_mechanics::{choose, effects, select};
use cantrip_oracle as oracle;

/// "As this creature enters, choose another creature you control" (CR 614.12a): a replacement effect that modifies how it
/// enters, not a trigger, so the choice is made before it enters and nobody can respond to it (the card's ruling). Among the
/// permanents its controller controls as it is about to enter: a creature entering at the same time is not on the battlefield
/// yet, and can't be chosen (the card's ruling). With no other creature, nothing is chosen.
fn choose_a_creature(g: &mut dyn Game, entering: &mut Entering) -> Result<(), Suspend> {
    let yours = g.permanents_of(entering.controller);
    let rule = Selection::new(select::as_many_as_can(1)).with(select::CREATURE).called("creature");
    choose::object_as_enters(g, entering, yours, rule)
}

/// "Sacrifice this creature: The chosen creature gains indestructible until end of turn." (CR 702.12, 611.2a): the creature
/// chosen as it entered, read as it last existed once sacrificed (CR 608.2h). If that creature has left the battlefield,
/// nothing gains indestructible, and none is chosen anew (the card's ruling; CR 400.7).
fn protect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = choose::chosen_object(g, g.linked(item)) else { return Ok(()) };
    effects::gain_until_end_of_turn(g, item.object, creature, Keyword::Indestructible);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enters: Some(choose_a_creature),
    abilities: &[ability(SACRIFICE_THIS, protect, "Sacrifice this creature: The chosen creature gains indestructible until end of turn.")],
    ..oracle::DAUNTLESS_BODYGUARD
};
