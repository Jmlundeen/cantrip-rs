use cantrip_api::*;
use cantrip_oracle as oracle;

/// The spell creates its delayed triggered ability with a stated duration (CR 603.7b): "whenever you cast a creature
/// spell this turn", which may trigger any number of times. "You" is the controller of the spell as it resolved
/// (CR 603.7d).
fn watch(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    cantrip_mechanics::delayed::this_turn(g, item, item.object);
    Ok(())
}

/// "Whenever you cast a creature spell": judged on the spell as it is on the stack (CR 601.2i), so a card that is a
/// creature only there counts.
fn you_cast_a_creature_spell(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::SpellCast { spell, player, .. } if *player == q.controller(source) && q.chars(*spell).is(CardType::Creature) => {
            Fired::PLAIN
        }
        _ => None,
    }
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], watch),
    delayed: &[triggered(when(EventMask::SPELL_CAST, you_cast_a_creature_spell), |g, item| cantrip_mechanics::effects::draw(g, item, 1), "Whenever you cast a creature spell this turn, draw a card.")],
    ..oracle::GLIMPSE_OF_NATURE
};
