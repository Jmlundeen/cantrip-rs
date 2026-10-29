use cantrip_api::*;
use cantrip_mechanics::sneak;
use cantrip_oracle as oracle;

/// "During your turn, you may cast creature spells with power or toughness 1 or less from your graveyard": a way a permanent
/// gives (CR 604, 601.3), a permission for the card's own mana cost (`GrantedWay::permission`), so an alternative cost may be
/// paid instead. The power and toughness are the spell's it would be, the face being cast, as `chars` gives them here (CR
/// 601.3e: Goring Warplow cast prototyped is a 1/1, CR 718.3a). It is not a limit of once, so the history is not asked.
fn cast_small_creature(q: &dyn Query, asker: &Asker, card: ObjectId, player: PlayerId) -> Option<Cost> {
    if player != asker.controller || q.state().turn.active != player {
        return None;
    }
    let c = q.chars(card);
    if !c.is(CardType::Creature) || !(c.power.is_some_and(|p| p <= 1) || c.toughness.is_some_and(|t| t <= 1)) {
        return None;
    }
    Some(Cost::mana(q.card(c.card).cost))
}

/// "If you cast a spell this way, that creature enters with a finality counter on it" (CR 614.1c, 122.6): a replacement
/// effect about the spell cast by the way (`GrantedWay::spell_replacement`), made as it is cast and so still applying if
/// Leonardo has left before it resolves. It applies as that spell enters the battlefield from the stack.
fn spell_enters(_: &dyn Query, _: &Asker, spell: ObjectId, proposal: &Proposal) -> bool {
    matches!(proposal, Proposal::ZoneChange(z)
        if z.object == spell && z.from.is(ZoneKind::Stack) && z.to.is(ZoneKind::Battlefield))
}

fn add_a_finality_counter(_: &dyn Query, _: &Asker, _: ObjectId, proposal: &mut Proposal) {
    if let Proposal::ZoneChange(z) = proposal {
        z.counters[CounterKind::Finality as usize] += 1;
    }
}

pub static CARD: CardDef = CardDef {
    ways: &[sneak::way(mana("{2}{W}{W}"), "Sneak {2}{W}{W}")],
    granted_ways: &[GrantedWay {
        text: "During your turn, you may cast creature spells with power or toughness 1 or less from your graveyard.",
        cost: cast_small_creature,
        permission: true,
        spell_replacement: Some(0),
        ..GrantedWay::CAST
    }],
    granted_replacements: &[granted_replacement(
        ReplacementKind::ZoneChange,
        spell_enters,
        add_a_finality_counter,
        "enters with a finality counter on it",
    )],
    ..oracle::LEONARDO_SEWER_SAMURAI
};
