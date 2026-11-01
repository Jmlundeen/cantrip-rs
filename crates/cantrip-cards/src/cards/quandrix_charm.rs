use cantrip_api::*;
use cantrip_mechanics::{destroy_target, effects, targets};
use cantrip_oracle as oracle;

/// "Destroy target enchantment." (CR 701.8): an indestructible one stays.
fn destroy_enchantment(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target(g, item, 0);
    Ok(())
}

/// "Target creature has base power and toughness 5/5 until end of turn." (CR 613.4b, 611.2a): layer 7b, so counters and
/// other modifiers apply on top; nothing if the creature is no longer legal (CR 608.2b).
fn base_five_five(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    g.add_effect(item.object, creature, EffectKind::SetPt(5, 5), Duration::EndOfTurn);
    Ok(())
}

static MODAL: ModalDef = modes(&[
    mode(effects::counter_first_target_unless_pays::<2>, "Counter target spell unless its controller pays {2}.").targets(&[targets::SPELL]),
    mode(destroy_enchantment, "Destroy target enchantment.").targets(&[targets::ENCHANTMENT]),
    mode(base_five_five, "Target creature has base power and toughness 5/5 until end of turn.").targets(&[targets::CREATURE]),
]);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::QUANDRIX_CHARM };
