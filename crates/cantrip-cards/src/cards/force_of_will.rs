use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

fn blue_card(q: &dyn Query, asker: &Asker, id: ObjectId) -> bool {
    q.chars(id).colors.has(q.words(asker.source).color(Color::Blue))
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::SPELL], effects::counter_first_target),
    // You may pay 1 life and exile a blue card from your hand rather than pay this spell's mana cost (CR 118.9).
    ways: &[way(life(1).and(exile_from_hand(1, blue_card)), ZoneKind::Hand, "Pay 1 life, exile a blue card from your hand")],
    ..oracle::FORCE_OF_WILL
};
