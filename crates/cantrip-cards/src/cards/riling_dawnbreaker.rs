use cantrip_api::*;
use cantrip_mechanics::tokens::{self, SOLDIER_2_2};
use cantrip_mechanics::{effects, targets, triggers::BEGINNING_OF_YOUR_COMBAT};
use cantrip_oracle as oracle;

/// "Create a 2/2 white Soldier creature token" (CR 701.7).
fn create_a_soldier(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &SOLDIER_2_2, 1)?;
    Ok(())
}

/// The Omen (CR 720): cast from the hand as an Omen sorcery for {1}{W}; as it resolves it is shuffled into its owner's
/// library (CR 720.3d, the core's).
pub static SIGNALING_ROAR: CardDef = CardDef { spell: spell(&[], create_a_soldier), ..oracle::SIGNALING_ROAR };

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        BEGINNING_OF_YOUR_COMBAT,
        effects::target_creature_gets::<1, 0>,
        "At the beginning of combat on your turn, another target creature you control gets +1/+0 until end of turn.",
    )
    .targets(&[targets::OTHER_CREATURE_YOU_CONTROL])],
    omen: Some(&SIGNALING_ROAR),
    ..oracle::RILING_DAWNBREAKER
};

pub static EXTRA: &[&CardDef] = &[&SIGNALING_ROAR];
