use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

/// "Whenever an opponent casts a spell with mana value 2 or less" (CR 601.2i, 603.2, 202.3): a spell cast (not a copy, CR 707.10a,
/// and not an ability) by a player other than this creature's controller, with the mana value it has on the stack, where an {X} in
/// its cost counts as the X chosen for it (CR 202.3e).
fn opponent_casts_a_spell_with_mana_value_2_or_less(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::SpellCast { spell, player, .. } if *player != q.controller(source) && q.mana_value(*spell) <= 2 => Fired::PLAIN,
        _ => None,
    }
}

/// "You gain 2 life."
fn gain_two(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::gain_life(g, item, 2)
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(when(EventMask::SPELL_CAST, opponent_casts_a_spell_with_mana_value_2_or_less), gain_two, "Whenever an opponent casts a spell with mana value 2 or less, you gain 2 life.")],
    ..oracle::SOLARIUM_SENTRY
};
