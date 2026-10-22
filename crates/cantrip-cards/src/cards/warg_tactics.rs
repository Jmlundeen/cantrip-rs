use cantrip_api::*;
use cantrip_mechanics::{destroy_target, effects::gain_hexproof_until_end_of_turn, targets, trample_until_end_of_turn};
use cantrip_oracle as oracle;

/// "Destroy target creature with flying." (CR 701.8; a target that has lost flying by then is illegal, CR 608.2b.)
fn destroy_flyer(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target(g, item, 0);
    Ok(())
}

/// "Put a +1/+1 counter on target creature you control. It gains trample and hexproof until end of turn." All of it only to a
/// target still legal as the spell resolves (CR 608.2b).
fn counter_trample_hexproof(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        g.add_counters(creature, CounterKind::PlusOne, 1);
        g.end_sentence();
        instruction(g, |g| {
            trample_until_end_of_turn(g, item.object, creature);
            gain_hexproof_until_end_of_turn(g, item.object, creature);
        });
    }
    Ok(())
}

static MODAL: ModalDef = modes(&[
    mode(destroy_flyer, "Destroy target creature with flying.").targets(&[targets::CREATURE_WITH_FLYING]),
    mode(counter_trample_hexproof, "Put a +1/+1 counter on target creature you control. It gains trample and hexproof until end of turn.")
        .targets(&[targets::CREATURE_YOU_CONTROL]),
]);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::WARG_TACTICS };
