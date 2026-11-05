use cantrip_api::*;
use cantrip_mechanics::targets::{self, filters};
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

/// The slot for one opponent of "for each opponent, gain control of up to one target creature that player controls with mana
/// value 3 or less" (CR 601.2c, 115.1): a creature that opponent controls, its mana value as the game reads it.
fn creature_the_slots_opponent_controls_with_mana_value_3_or_less(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    let Target::Object(id) = t else { return false };
    targets::controlled_by_slot_opponent(q, id) && filters::creature(q, asker, id) && q.mana_value(id) <= 3
}

const FOR_EACH_OPPONENT: TargetSpec = target(creature_the_slots_opponent_controls_with_mana_value_3_or_less)
    .between(0, TargetSpec::EACH_OPPONENT)
    .labelled("Creature {player} controls with mana value 3 or less");

/// "Until end of turn, they become Warriors in addition to their other types and gain haste." (CR 613.1d, 611.2a, 702.10):
/// one instruction, so a single effect gives both to each creature taken.
fn become_warriors_with_haste(g: &mut dyn Game, item: &StackItem, creatures: &[ObjectId]) {
    instruction(g, |g| {
        for &creature in creatures {
            cantrip_mechanics::effects::becomes_subtype_until_end_of_turn(g, item.object, creature, Subtype::Warrior);
            cantrip_mechanics::haste_until_end_of_turn(g, item.object, creature);
        }
    });
}

/// "Gain control of up to one target creature that player controls with mana value 3 or less until end of turn. Untap those
/// creatures. Until end of turn, they become Warriors in addition to their other types and gain haste." (CR 613.1b, 608.2b):
/// each target still legal is taken, all at once, then untapped together, then given the types and haste.
fn take_them(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let creatures = targets::legal_objects(g, item);
    // One instruction (CR 611.2c): the same effect takes every creature the ability chose, not one instruction each.
    instruction(g, |g| {
        for &creature in &creatures {
            g.gain_control(item.object, creature, item.controller, Duration::EndOfTurn);
        }
    });
    g.end_sentence();
    for &creature in &creatures {
        g.untap(creature);
    }
    g.end_sentence();
    become_warriors_with_haste(g, item, &creatures);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::THIS_ENTERS,
        take_them,
        "When Worf enters, for each opponent, gain control of up to one target creature that player controls with mana value 3 or less until end of turn. Untap those creatures. Until end of turn, they become Warriors in addition to their other types and gain haste.",
    )
    .targets(&[FOR_EACH_OPPONENT])],
    ..oracle::WORF_INTELLIGENCE_CONTRACTOR
};
