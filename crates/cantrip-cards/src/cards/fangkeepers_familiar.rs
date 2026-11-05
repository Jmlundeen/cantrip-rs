use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{counter_target, effects, targets};
use cantrip_oracle as oracle;

/// "You gain 3 life and surveil 3" (CR 119.1, 701.25a): in that order.
fn gain_3_and_surveil_3(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::gain_life(g, item, 3)?;
    effects::surveil(g, item, 3)
}

/// "Counter target creature spell" (CR 701.6).
fn counter_creature_spell(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    counter_target(g, item, 0);
    Ok(())
}

static MODAL: ModalDef = modes(&[
    mode(gain_3_and_surveil_3, "You gain 3 life and surveil 3."),
    mode(effects::destroy_first_target, "Destroy target enchantment.").targets(&[targets::ENCHANTMENT]),
    mode(counter_creature_spell, "Counter target creature spell.").targets(&[targets::CREATURE_SPELL]),
]);

pub static CARD: CardDef = CardDef {
    abilities: &[modal_triggered(
        THIS_ENTERS,
        &MODAL,
        "When this creature enters, choose one \u{2014} You gain 3 life and surveil 3; or destroy target enchantment; or counter target creature spell.",
    )],
    ..oracle::FANGKEEPERS_FAMILIAR
};
