use cantrip_api::*;
use cantrip_mechanics::{damage_target, effects, targets, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// "Target creature without flying" (CR 115.1, 702.9): judged on the creature's keywords as it is chosen and again as the
/// spell resolves (CR 608.2b), so a creature that gains flying in response is no longer a legal target.
fn creature_without_flying(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    (targets::CREATURE.filter)(q, asker, t) && matches!(t, Target::Object(id) if !q.chars(id).has(Keyword::Flying))
}

/// "Charring Bite deals 5 damage to target creature without flying."
fn charring_bite(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 5);
    Ok(())
}

/// The Omen (CR 720): cast from the hand as an Omen sorcery for {1}{R}; as it resolves it is shuffled into its owner's
/// library (CR 720.3d, the core's).
pub static CHARRING_BITE: CardDef = CardDef {
    spell: spell(&[target(creature_without_flying)], charring_bite),
    ..oracle::CHARRING_BITE
};

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, |g, item| effects::gain_life(g, item, 5), "When this creature enters, you gain 5 life.")],
    omen: Some(&CHARRING_BITE),
    ..oracle::TWINMAW_STORMBROOD
};

pub static EXTRA: &[&CardDef] = &[&CHARRING_BITE];
