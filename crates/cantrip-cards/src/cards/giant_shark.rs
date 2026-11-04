use cantrip_api::*;
use cantrip_mechanics::{effects, history, lands, pump, trample_until_end_of_turn, triggers};
use cantrip_oracle as oracle;

/// "Whenever this creature blocks or becomes blocked by a creature that has been dealt damage this turn" (CR 603.2, 509.3b, 120.2).
fn blocks_or_becomes_blocked_by_damaged_creature(q: &dyn Query, asker: &Asker, event: &Event) -> Option<Fired> {
    let other = triggers::other_creature_in_block_with_this(q, asker.source, event)?;
    if history::was_dealt_damage_this_turn(q.history(), other) {
        return Fired::about(Target::Object(other));
    }
    None
}

/// "This creature gets +2/+0 and gains trample until end of turn." (CR 608.2, 611.2a).
fn gets_2_0_and_gains_trample(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    instruction(g, |g| {
        pump(g, item.object, item.object, 2, 0);
        trample_until_end_of_turn(g, item.object, item.object);
    });
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            when_asked(EventMask::BLOCKS, blocks_or_becomes_blocked_by_damaged_creature),
            gets_2_0_and_gains_trample,
            "Whenever this creature blocks or becomes blocked by a creature that has been dealt damage this turn, this creature gets +2/+0 and gains trample until end of turn.",
        ),
        state_triggered(
            lands::you_control_no_islands,
            effects::sacrifice_self,
            "When you control no Islands, sacrifice this creature.",
        ),
    ],
    attack_rules: &[lands::attack_rule_defender_controls::<{ lands::sub(Subtype::Island) }>],
    ..oracle::GIANT_SHARK
};
