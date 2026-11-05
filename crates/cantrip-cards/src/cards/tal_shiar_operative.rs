use cantrip_api::*;
use cantrip_mechanics::{effects, targets, triggers::THIS_ATTACKS};
use cantrip_oracle as oracle;

fn target_gains_deathtouch(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        effects::gain_until_end_of_turn(g, item.object, creature, Keyword::Deathtouch);
    }
    Ok(())
}

fn surveil_one(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::surveil(g, item, 1)
}

static MODAL: ModalDef = modes(&[
    mode(target_gains_deathtouch, "Target creature you control gains deathtouch until end of turn.")
        .targets(&[targets::CREATURE_YOU_CONTROL]),
    mode(surveil_one, "Surveil 1."),
]);

pub static CARD: CardDef = CardDef {
    abilities: &[modal_triggered(
        THIS_ATTACKS,
        &MODAL,
        "Whenever this creature attacks, choose one — target creature you control gains deathtouch until end of turn; or surveil 1.",
    )],
    ..oracle::TAL_SHIAR_OPERATIVE
};
