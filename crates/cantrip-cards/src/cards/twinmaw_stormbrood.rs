use cantrip_api::*;
use cantrip_mechanics::{effects, targets, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// The Omen (CR 720): cast from the hand as an Omen sorcery for {1}{R}; as it resolves it is shuffled into its owner's
/// library (CR 720.3d, the core's).
pub static CHARRING_BITE: CardDef = CardDef {
    spell: spell(&[targets::CREATURE_WITHOUT_FLYING], effects::damage_first_target::<5>),
    ..oracle::CHARRING_BITE
};

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, |g, item| effects::gain_life(g, item, 5), "When this creature enters, you gain 5 life.")],
    omen: Some(&CHARRING_BITE),
    ..oracle::TWINMAW_STORMBROOD
};

pub static EXTRA: &[&CardDef] = &[&CHARRING_BITE];
