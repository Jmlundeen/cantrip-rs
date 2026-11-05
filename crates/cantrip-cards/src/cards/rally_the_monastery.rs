use cantrip_api::*;
use cantrip_mechanics::{cost_reduction, effects, targets, tokens};
use cantrip_oracle as oracle;

/// "Create two 1/1 white Monk creature tokens with prowess" (CR 111.1, 702.108).
fn create_two_monks(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::MONK, 2)?;
    Ok(())
}

static MODAL: ModalDef = modes(&[
    mode(create_two_monks, "Create two 1/1 white Monk creature tokens with prowess."),
    mode(effects::targets_get::<2, 2>, "Up to two target creatures you control each get +2/+2 until end of turn.")
        .targets(&[targets::CREATURE_YOU_CONTROL.between(0, 2)]),
    mode(effects::destroy_first_target, "Destroy target creature with power 4 or greater.").targets(&[targets::CREATURE_WITH_POWER_4_OR_GREATER]),
]);

/// The Monk token the first mode makes.
pub static EXTRA: &[&CardDef] = &[&tokens::MONK];

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef::MODAL),
    modal: Some(&MODAL),
    own_cost_changes: &[cost_reduction::less_if_you_cast_another_spell_this_turn::<2>],
    ..oracle::RALLY_THE_MONASTERY
};
