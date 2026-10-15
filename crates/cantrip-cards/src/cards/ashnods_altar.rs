use cantrip_api::*;
use cantrip_mechanics::mana;
use cantrip_oracle as oracle;

fn creature_you_control(q: &dyn Query, asker: &Asker, id: ObjectId) -> bool {
    q.chars(id).is(CardType::Creature) && q.controller(id) == asker.controller
}

/// "Sacrifice a creature: Add {C}{C}." A mana ability whose sacrifice comes with the activation (Phyrexian Altar's
/// shape).
pub static CARD: CardDef = CardDef {
    abilities: &[AbilityDef::Activated(ActivatedDef {
        text: "Sacrifice a creature: Add {C}{C}.",
        cost: Cost::sacrifice(1, creature_you_control),
        mana: Some(mana::add_cc),
        adds: ManaPool([0, 0, 0, 0, 0, 2]),
        ..ActivatedDef::BLANK
    })],
    ..oracle::ASHNODS_ALTAR
};
