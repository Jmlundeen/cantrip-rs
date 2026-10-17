use cantrip_api::*;
use cantrip_mechanics::firebending;
use cantrip_oracle as oracle;

/// Whether `card` is one of the cards "in your graveyard" "during your turn" whose caster is Iroh's controller.
fn yours_on_your_turn(q: &dyn Query, asker: &Asker, card: ObjectId, caster: PlayerId) -> bool {
    let you = asker.controller;
    caster == you && q.state().turn.active == you && q.obj(card).owner == you
}

/// Whether the card in the graveyard has a quality: the card itself, never the face it would be cast as, since a card has
/// only its normal characteristics there (CR 715.4: Bonecrusher Giant is a creature card, though Stomp is an instant), and a
/// split card has both halves' (CR 709.4: Leave // Chance is an instant and sorcery card).
fn card_is(q: &dyn Query, card: ObjectId, quality: impl Fn(&CardDef) -> bool) -> bool {
    let printed = q.card(q.obj(card).card);
    match printed.split {
        Some((left, right)) => quality(left) || quality(right),
        None => quality(printed),
    }
}

fn instant_or_sorcery(def: &CardDef) -> bool {
    def.is(CardType::Instant) || def.is(CardType::Sorcery)
}

fn lesson(def: &CardDef) -> bool {
    def.subtypes.has(Subtype::Lesson)
}

/// "During your turn, each non-Lesson instant and sorcery card in your graveyard has flashback. The flashback cost is
/// equal to that card's mana cost" (CR 702.34a, 113.6, 604). A card with no mana cost has a flashback cost that can't be
/// paid (CR 118.6), so it is not cast this way.
/// The cost is the mana cost of the face cast (a split card's half, CR 709.3a), which `q` reads the spell as.
fn flashback_for_its_mana_cost(q: &dyn Query, asker: &Asker, card: ObjectId, caster: PlayerId) -> Option<Cost> {
    let def = q.card(q.chars(card).card);
    (yours_on_your_turn(q, asker, card, caster) && card_is(q, card, instant_or_sorcery) && !card_is(q, card, lesson) && def.has_cost)
        .then(|| Cost::mana(def.cost))
}

/// "During your turn, each Lesson card in your graveyard has flashback {1}" (CR 702.34a).
fn flashback_for_one(q: &dyn Query, asker: &Asker, card: ObjectId, caster: PlayerId) -> Option<Cost> {
    (yours_on_your_turn(q, asker, card, caster) && card_is(q, card, lesson)).then(|| Cost::mana(ManaCost { generic: 1, ..ManaCost::ZERO }))
}

pub static CARD: CardDef = CardDef {
    abilities: &[firebending::trigger::<2>("Firebending 2")],
    granted_ways: &[
        GrantedWay { text: "Flashback equal to its mana cost", cost: flashback_for_its_mana_cost, flashback: true, ..GrantedWay::CAST },
        GrantedWay { text: "Flashback {1}", cost: flashback_for_one, flashback: true, ..GrantedWay::CAST },
    ],
    ..oracle::IROH_GRAND_LOTUS
};
