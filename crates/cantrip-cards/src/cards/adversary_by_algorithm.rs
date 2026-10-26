use cantrip_api::*;
use cantrip_mechanics::{beam_me_up, tokens};
use cantrip_oracle as oracle;

/// "Create a 3/3 colorless Illusion artifact creature token."
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::ILLUSION_3_3_ARTIFACT, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], effect),
    ways: &[beam_me_up::way(mana("{3}{G}"), "Beam me up {3}{G}")],
    ..oracle::ADVERSARY_BY_ALGORITHM
};

pub static EXTRA: &[&CardDef] = &[&tokens::ILLUSION_3_3_ARTIFACT];
