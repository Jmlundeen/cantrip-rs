use cantrip_api::*;
use cantrip_mechanics::{replacements, sneak};
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

pub static CARD: CardDef = CardDef {
    ways: &[sneak::way(mana("{2}{W}{W}"), "Sneak {2}{W}{W}")],
    granted_ways: &[GrantedWay {
        text: "During your turn, you may cast creature spells with power or toughness 1 or less from your graveyard.",
        cost: cast_small_creature,
        permission: true,
        spell_replacement: Some(0),
        ..GrantedWay::CAST
    }],
    // "If you cast a spell this way, that creature enters with a finality counter on it": a replacement effect about the
    // spell cast by the way (`GrantedWay::spell_replacement`), made as it is cast and so still applying if Leonardo has
    // left before it resolves.
    granted_replacements: &[replacements::SPELL_ENTERS_WITH_FINALITY
        .text("If you cast a spell this way, that creature enters with a finality counter on it.")],
    ..oracle::LEONARDO_SEWER_SAMURAI
};
