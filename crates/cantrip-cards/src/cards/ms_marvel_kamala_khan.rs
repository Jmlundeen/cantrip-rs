use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Whenever you cast a spell that targets a creature you control" (CR 603.2, 601.2c): a question about the spell as it is
/// cast, whose targets are public from the moment it was announced. One trigger for the spell however many of the
/// creatures it targets (CR 603.2c is about events: one cast is one event). A copy is not cast (CR 707.10), so it sets
/// nothing off; a spell that targets only a creature an opponent controls, a player, or a card elsewhere does not.
fn you_cast_a_spell_that_targets_your_creature(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    if let Event::SpellCast { spell, player, .. } = event
        && *player == q.controller(source)
        && let Some(item) = q.state().stack.iter().find(|i| i.kind == StackKind::Spell && i.object == *spell)
        && item.targets().iter().filter_map(|t| t.object()).any(|id| {
            targets::permanent(q, id) && q.chars(id).is(CardType::Creature) && q.controller(id) == *player
        })
    {
        Fired::PLAIN
    } else {
        None
    }
}

/// "Draw a card. Until end of turn, Ms. Marvel gains 'Ms. Marvel's base power is equal to the number of cards in your hand.'"
/// (CR 611.2a, 613.1f): the static ability is given to the object that is Ms. Marvel now, and only to it; a Ms. Marvel that
/// has left the battlefield is a new object the effect knows nothing of (CR 400.7). The card is her own, whatever she
/// has become a copy of, as the granted ability is one of her card's.
fn embiggen_fist(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.draw(item.controller, 1);
    let me = g.card_id(&CARD);
    g.add_effect(item.object, item.object, EffectKind::AddAbility(AbilityRef::delayed(me, 0)), Duration::EndOfTurn);
    Ok(())
}

/// "Ms. Marvel's base power is equal to the number of cards in your hand" (CR 613.4b, layer 7b): an effect that refers to
/// the base power sets it, read live as the hand changes (CR 611.3a). The source is the object that has the ability, so
/// "your" is its controller's now (CR 109.5). +1/+1 counters and other modifiers apply on top, in layer 7c.
fn base_power_is_cards_in_hand(q: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    let state = q.state();
    chars.power = Some(state.zone(Zone::hand(state.obj(source).controller)).len() as i16);
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when(EventMask::SPELL_CAST, you_cast_a_spell_that_targets_your_creature),
        embiggen_fist,
        "Embiggen Fist \u{2014} Whenever you cast a spell that targets a creature you control, draw a card. Until end of turn, Ms. Marvel gains \"Ms. Marvel's base power is equal to the number of cards in your hand.\"",
    )],
    delayed: &[granted_static(Layer::PtSet, base_power_is_cards_in_hand, "Ms. Marvel's base power is equal to the number of cards in your hand.")],
    play_rules: &[PlayRule::NoMaximumHandSize],
    ..oracle::MS_MARVEL_KAMALA_KHAN
};
