use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Put a +1/+1 counter on it."
fn counter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Target::Object(creature) = item.subject {
        g.add_counters(creature, CounterKind::PlusOne, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[AbilityDef::Triggered(TriggeredDef {
        text: "Whenever a creature attacks enchanted player, put a +1/+1 counter on it.",
        listens: EventMask::ATTACKS,
        matches: cantrip_mechanics::triggers::enchanted_player_attacked,
        effect: counter,
        ..TriggeredDef::BLANK
    })],
    enchant: Some(targets::PLAYER),
    ..oracle::CURSE_OF_PREDATION
};
