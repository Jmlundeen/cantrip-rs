use cantrip_api::*;
use cantrip_mechanics::craft;
use cantrip_mechanics::effects;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

/// "Dinosaurs": a Dinosaur you control or a Dinosaur card in your graveyard (CR 702.167b), the creature type read through
/// the words of the Lattice's text (CR 612).
fn dinosaur(q: &dyn Query, asker: &Asker, id: ObjectId) -> bool {
    q.chars(id).has_subtype(q.words(asker.source).subtype(Subtype::Dinosaur))
}

/// "Craft with one or more Dinosaurs."
static DINOSAURS: Materials = craft::materials(1, true, dinosaur, "Dinosaur");

pub static BACK: CardDef = CardDef {
    statics: &[characteristic_defining(Layer::PtDefine, craft::power_of_total_crafted_power, "Mastercraft Raptor's power is equal to the total power of the exiled cards used to craft it.").lost_pt(0, 4)],
    ..oracle::MASTERCRAFT_RAPTOR
};

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, |g, item| effects::may_discard_to_draw(g, item, 2), "When this artifact enters, you may discard a card. If you do, draw two cards."),
        craft::ability(mana("{4}{R}"), &DINOSAURS, "Craft with one or more Dinosaurs {4}{R}"),
    ],
    back: Some(&BACK),
    ..oracle::SAHEELIS_LATTICE
};
