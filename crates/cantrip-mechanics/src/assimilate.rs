//! Assimilate a card as the printed reminder defines it: put the creature card onto the battlefield under the ability
//! controller's control with a +1/+1 counter; it is a Borg artifact creature and loses its other creature types. Under
//! CR 205.1b, "artifact creature" retains prior card types and subtypes other than creature types; the Borg subtype replaces
//! its prior creature subtypes. Assimilation Assessor, Borg Cube, Borg Queen, Perfection Manifest, Death Is Irrelevant,
//! Assimilate the Wounded, Gift of Perfection, The Best of Both Worlds, and The Borg Queen print this action.

use cantrip_api::*;

static AS_BORG_ARTIFACT_CREATURE: EntersAs = EntersAs {
    adds_types: TypeSet::of(&[CardType::Artifact, CardType::Creature]),
    loses_subtypes_of: &[CardType::Creature],
    subtypes: &[Subtype::Borg],
    ..EntersAs::NONE
};

/// Assimilate `card` as it enters: add artifact and creature types, replace its creature types with Borg, and add the
/// +1/+1 counter before entry replacements and ETB triggers (CR 205.1b, 400.7, 614.1c, 614.12). `None` means the effect
/// could not put it onto the battlefield.
pub fn assimilate(g: &mut dyn Game, item: &StackItem, card: ObjectId) -> Result<Option<ObjectId>, Suspend> {
    Ok(assimilate_all(g, item, &[card])?.first().copied())
}

/// Assimilate several cards at once (CR 101.4, 608.2f): they are put onto the battlefield as one event, each with its
/// +1/+1 counter and as a Borg artifact creature, so each entering permanent sees the others already there (CR 614.12).
/// Assimilate the Wounded's "for each opponent, assimilate up to one target creature card" prints it. `None` cards are
/// not a case: the caller passes only the targets still legal as it resolves (CR 608.2b).
pub fn assimilate_all(g: &mut dyn Game, item: &StackItem, cards: &[ObjectId]) -> Result<Vec<ObjectId>, Suspend> {
    let entering: Vec<(ObjectId, PlayerId)> = cards.iter().map(|&card| (card, item.controller)).collect();
    g.put_all_onto_battlefield_becoming(&entering, item, &AS_BORG_ARTIFACT_CREATURE, &[(CounterKind::PlusOne, 1)])
}

/// "Assimilate target creature card ...": the ability's first target, assimilated as the ability resolves; nothing happens if
/// the target is no longer legal (CR 608.2b). Borg Queen, Perfection Manifest and Death Is Irrelevant print it.
pub fn assimilate_the_first_target(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(card) = crate::targets::object(g, item, 0) else { return Ok(()) };
    assimilate(g, item, card)?;
    Ok(())
}
