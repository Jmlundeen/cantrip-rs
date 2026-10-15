use cantrip_api::*;
use cantrip_mechanics::look_for_creature_or_land;
use cantrip_oracle as oracle;

/// Look at the top five cards of your library. You may reveal a creature or land card from among them and put it into your
/// hand. Put the rest on the bottom of your library in a random order.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    look_for_creature_or_land(g, item.controller, item.object, 5)
}

/// "If this spell is the first spell you've cast this game, you may cast it without paying its mana cost" (CR 118.9): read
/// at announcement, before the spell asking is counted (CR 601.2i).
fn first_spell(q: &dyn Query, asker: &Asker) -> bool {
    q.history().casts_this_game(asker.controller) == 0
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[], effect }),
    ways: &[WayDef {
        text: "Without paying its mana cost",
        cost: Cost::NONE,
        from: ZoneKind::Hand,
        exile_after: false,
        condition: Some(first_spell),
        instant_speed: false,
    }],
    ..oracle::ONCE_UPON_A_TIME
};
