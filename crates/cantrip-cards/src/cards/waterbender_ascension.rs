use cantrip_api::*;
use cantrip_mechanics::{effects, targets, triggers};
use cantrip_oracle as oracle;

/// "Put a quest counter on this enchantment. Then if it has four or more quest counters on it, draw a card." Not an
/// intervening "if" (CR 603.4): the count is read once the counter has been put on, so the fourth counter and every one
/// after it draw, and a count short of four draws nothing. An enchantment that has left the battlefield gets no counter
/// (CR 122.6) and is read as it last was (CR 608.2h).
fn quest_counter_then_draw(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_counters(item.object, CounterKind::Quest, 1);
    if g.obj(item.object).counters(CounterKind::Quest) >= 4 {
        effects::draw(g, item, 1)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            when_asked(EventMask::DAMAGE, triggers::creature_you_control_deals_combat_damage_to_player),
            quest_counter_then_draw,
            "Whenever a creature you control deals combat damage to a player, put a quest counter on this enchantment. Then if it has four or more quest counters on it, draw a card.",
        ),
        ability(
            cost("{4}"),
            effects::target_creature_cant_be_blocked_this_turn,
            "Waterbend {4}: Target creature can't be blocked this turn. (While paying a waterbend cost, you can tap your artifacts and creatures to help. Each one pays for {1}.)",
        )
        .waterbend()
        .targets(&[targets::CREATURE]),
    ],
    ..oracle::WATERBENDER_ASCENSION
};
