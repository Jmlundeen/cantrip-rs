use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "Target creature gains indestructible until end of turn." Nothing happens if the target is no longer legal (CR 608.2b).
fn gain_indestructible(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        effects::gain_indestructible_until_end_of_turn(g, item.object, creature);
    }
    Ok(())
}

static MODAL: ModalDef = modes(&[
    mode(gain_indestructible, "Target creature gains indestructible until end of turn.").targets(&[targets::CREATURE]),
    mode(effects::destroy_first_target, "Destroy target creature with toughness 4 or greater.").targets(&[targets::CREATURE_WITH_TOUGHNESS_4_OR_GREATER]),
]);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::VALOROUS_STANCE };
