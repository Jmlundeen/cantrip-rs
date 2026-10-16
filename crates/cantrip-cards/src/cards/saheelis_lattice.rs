use cantrip_api::*;
use cantrip_mechanics::craft;
use cantrip_mechanics::effects::may_discard_to_draw_two;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

/// "Dinosaurs": a Dinosaur you control or a Dinosaur card in your graveyard (CR 702.167b), the creature type read through
/// the words of the Lattice's text (CR 612).
fn dinosaur(q: &dyn Query, asker: &Asker, id: ObjectId) -> bool {
    q.chars(id).has_subtype(q.words(asker.source).subtype(Subtype::Dinosaur))
}

/// "Craft with one or more Dinosaurs."
static DINOSAURS: Materials = craft::materials(1, true, dinosaur, "Dinosaur");

/// "Mastercraft Raptor's power is equal to the total power of the exiled cards used to craft it" (CR 604.3, 702.167c): each
/// card's power as it is in exile, a characteristic-defining ability of its own included (the card's ruling), none counting
/// for a card with no power. With none used to craft it, its power is 0.
fn total_crafted_power(q: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    let total: i16 = craft::used_to_craft(q.state(), source).into_iter().map(|c| q.chars(c).power.unwrap_or(0)).sum();
    chars.power = Some(total);
}

pub static BACK: CardDef = CardDef {
    statics: &[characteristic_defining(Layer::PtDefine, total_crafted_power, "Mastercraft Raptor's power is equal to the total power of the exiled cards used to craft it.").lost_pt(0, 4)],
    ..oracle::MASTERCRAFT_RAPTOR
};

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, may_discard_to_draw_two, "When this artifact enters, you may discard a card. If you do, draw two cards."),
        craft::ability(mana("{4}{R}"), &DINOSAURS, "Craft with one or more Dinosaurs {4}{R}"),
    ],
    back: Some(&BACK),
    ..oracle::SAHEELIS_LATTICE
};
