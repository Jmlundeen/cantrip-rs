use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::triggers::CREATURE_SPELL_YOU_CAST;
use cantrip_mechanics::{targets, web_slinging};
use cantrip_oracle as oracle;

/// Pavitr's Sevā: "Put a +1/+1 counter on target creature you control. It gains flying until end of turn." (CR 122.1,
/// 611.2)
fn seva(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        g.add_counters(creature, CounterKind::PlusOne, 1);
        effects::gain_until_end_of_turn(g, item.object, creature, Keyword::Flying);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    ways: &[web_slinging::way(mana("{1}{G}{W}"), "Web-slinging {1}{G}{W}")],
    abilities: &[triggered(
        CREATURE_SPELL_YOU_CAST,
        seva,
        "Pavitr's Sev\u{0101} \u{2014} Whenever you cast a creature spell, put a +1/+1 counter on target creature you control. It gains flying until end of turn.",
    )
    .targets(&[targets::CREATURE_YOU_CONTROL])],
    ..oracle::SPIDER_MAN_INDIA
};
