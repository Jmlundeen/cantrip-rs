use cantrip_api::*;
use cantrip_oracle as oracle;

fn creature_you_control(q: &dyn Query, asker: &Asker, id: ObjectId) -> bool {
    q.chars(id).is(CardType::Creature) && q.controller(id) == asker.controller
}

/// "Sacrifice a creature: Add {C}{C}." A mana ability whose sacrifice comes with the activation (Phyrexian Altar's
/// shape).
pub static CARD: CardDef = CardDef {
    abilities: &[mana_ability(Cost::sacrifice(1, creature_you_control), add_n(Mana::C, 2), "Sacrifice a creature: Add {C}{C}.")],
    ..oracle::ASHNODS_ALTAR
};
