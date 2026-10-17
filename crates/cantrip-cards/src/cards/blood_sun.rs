use cantrip_api::*;
use cantrip_mechanics::{effects, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// "All lands": every land on the battlefield, whoever controls it, as layers 1 to 5 made it (CR 613.1f). Land cards
/// elsewhere are not affected (the ruling).
fn all_lands(_: &dyn LayerQuery, _: ObjectId, _: ObjectId, so_far: &Chars) -> bool {
    so_far.is(CardType::Land)
}

pub static CARD: CardDef = CardDef {
    // "When this enchantment enters, draw a card." (CR 603.6a)
    abilities: &[triggered(THIS_ENTERS, |g, item| effects::draw(g, item, 1), "When this enchantment enters, draw a card.")],
    // "All lands lose all abilities except mana abilities." (CR 613.1f, 605.1)
    statics: &[loses_all_abilities_except_mana_abilities(all_lands, "All lands lose all abilities except mana abilities.")],
    ..oracle::BLOOD_SUN
};
