use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

/// "Whenever you cast a spell with mana value 4 or greater" (CR 603.2, 601.2i, 202.3): a spell cast, not copied (CR 707.10), by
/// this creature's controller, with the mana value it has on the stack - the X chosen for it counts (CR 202.3e).
fn you_cast_a_spell_with_mana_value_4_or_greater(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match event {
        Event::SpellCast { spell, player, .. } if *player == me.controller && q.mana_value(*spell) >= 4 => Fired::PLAIN,
        _ => None,
    }
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add_one_of(&[Color::Red, Color::Green]), "{T}: Add {R} or {G}."),
        triggered(
            when_asked(EventMask::SPELL_CAST, you_cast_a_spell_with_mana_value_4_or_greater),
            effects::untap_self,
            "Whenever you cast a spell with mana value 4 or greater, untap this creature.",
        ),
    ],
    ..oracle::SPIDER_MANIFESTATION
};
