use cantrip_api::*;
use cantrip_mechanics::{discard_chosen, effects, triggers};
use cantrip_oracle as oracle;

/// "Discard a card. If you do, draw a card" (CR 701.9, 608.2c): the controller chooses a card of their hand and discards it, an
/// empty hand being asked nothing; the draw follows a card actually discarded.
fn discard_to_draw(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if !discard_chosen(g, item.controller, item.object, 1)?.is_empty() {
        g.draw(item.controller, 1);
    }
    Ok(())
}

/// "Whenever you play a land or cast a spell from anywhere other than your hand" (CR 601.2a, 305.1, 603.2): a land its controller
/// played (the zone change of CR 305.1's play, from any zone but their hand), or a spell they cast, from any zone but their hand - the graveyard, exile, the top of the library. A copy of a spell
/// is not cast, so it does not trigger.
fn you_play_a_land_or_cast_a_spell_from_outside_your_hand(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match event {
        Event::SpellCast { spell, player, .. }
            if *player == me.controller && q.obj(*spell).cast_from.is_some_and(|zone| zone != ZoneKind::Hand) =>
        {
            Fired::PLAIN
        }
        Event::ZoneChange { new, from, reason: ZoneReason::PlayLand, .. } if q.obj(*new).controller == me.controller && *from != Zone::hand(me.controller) => Fired::PLAIN,
        _ => None,
    }
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::YOUR_FIRST_MAIN_PHASE,
            discard_to_draw,
            "Unreliable Visions \u{2014} At the beginning of your first main phase, discard a card. If you do, draw a card.",
        ),
        triggered(
            when_asked(EventMask(EventMask::SPELL_CAST.0 | EventMask::ZONE_CHANGE.0), you_play_a_land_or_cast_a_spell_from_outside_your_hand),
            |g, item| {
                effects::damage_each_opponent(g, item, 1);
                Ok(())
            },
            "Undying Vengeance \u{2014} Whenever you play a land or cast a spell from anywhere other than your hand, this enchantment deals 1 damage to each opponent.",
        ),
    ],
    ..oracle::SHADOW_OF_THE_GOBLIN
};
