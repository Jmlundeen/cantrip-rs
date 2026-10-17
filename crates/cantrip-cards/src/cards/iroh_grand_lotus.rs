use cantrip_api::*;
use cantrip_mechanics::firebending;
use cantrip_oracle as oracle;

/// Whether `card` is one of the cards "in your graveyard" "during your turn" whose caster is Iroh's controller, and an
/// instant or sorcery card (read as the face that would be cast, CR 709.3a).
fn yours_on_your_turn(q: &dyn Query, asker: &Asker, card: ObjectId, caster: PlayerId) -> Option<Chars> {
    let you = asker.controller;
    if caster != you || q.state().turn.active != you || q.obj(card).owner != you {
        return None;
    }
    Some(q.chars(card))
}

/// "During your turn, each non-Lesson instant and sorcery card in your graveyard has flashback. The flashback cost is
/// equal to that card's mana cost" (CR 702.34a, 113.6, 604). A card with no mana cost has a flashback cost that can't be
/// paid (CR 118.6), so it is not cast this way.
fn flashback_for_its_mana_cost(q: &dyn Query, asker: &Asker, card: ObjectId, caster: PlayerId) -> Option<Cost> {
    let chars = yours_on_your_turn(q, asker, card, caster)?;
    let def = q.card(chars.card);
    let spell = chars.is(CardType::Instant) || chars.is(CardType::Sorcery);
    (spell && !chars.has_subtype(Subtype::Lesson) && def.has_cost).then(|| Cost::mana(def.cost))
}

/// "During your turn, each Lesson card in your graveyard has flashback {1}" (CR 702.34a).
fn flashback_for_one(q: &dyn Query, asker: &Asker, card: ObjectId, caster: PlayerId) -> Option<Cost> {
    let chars = yours_on_your_turn(q, asker, card, caster)?;
    chars.has_subtype(Subtype::Lesson).then(|| Cost::mana(ManaCost { generic: 1, ..ManaCost::ZERO }))
}

pub static CARD: CardDef = CardDef {
    abilities: &[firebending::trigger::<2>("Firebending 2")],
    granted_ways: &[
        GrantedWay { text: "Flashback equal to its mana cost", cost: flashback_for_its_mana_cost, flashback: true, ..GrantedWay::CAST },
        GrantedWay { text: "Flashback {1}", cost: flashback_for_one, flashback: true, ..GrantedWay::CAST },
    ],
    ..oracle::IROH_GRAND_LOTUS
};
