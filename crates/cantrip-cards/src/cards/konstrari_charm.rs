use cantrip_api::*;
use cantrip_mechanics::{damage_target, targets, trample_until_end_of_turn};
use cantrip_oracle as oracle;

/// "Konstrari Charm deals 6 damage to target creature with flying": nothing if the target is gone or no longer has flying as it
/// resolves (CR 608.2b).
fn six_damage(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 6);
    Ok(())
}

/// "Put two +1/+1 counters on target creature. It gains trample until end of turn." Both stop if the target is no longer legal
/// (CR 608.2b).
fn counters_and_trample(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        g.add_counters(creature, CounterKind::PlusOne, 2);
        trample_until_end_of_turn(g, item.object, creature);
    }
    Ok(())
}

/// "Add {C}{C}{C}." (CR 106.4): a spell's effect, not a mana ability, so it uses the stack.
fn add_colorless(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_mana(item.controller, Mana::C, 3);
    Ok(())
}

static MODAL: ModalDef = modes(&[
    mode(six_damage, "Konstrari Charm deals 6 damage to target creature with flying.").targets(&[targets::CREATURE_WITH_FLYING]),
    mode(counters_and_trample, "Put two +1/+1 counters on target creature. It gains trample until end of turn.").targets(&[targets::CREATURE]),
    mode(add_colorless, "Add {C}{C}{C}."),
]);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::KONSTRARI_CHARM };
