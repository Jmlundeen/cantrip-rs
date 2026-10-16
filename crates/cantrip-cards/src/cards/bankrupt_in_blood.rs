use cantrip_api::*;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    spell: spell(&[], |g, item| cantrip_mechanics::effects::draw(g, item, 3)),
    // As an additional cost to cast this spell, sacrifice two creatures (CR 118.8).
    additional_cost: Some(Cost::sacrifice(2, cantrip_mechanics::targets::filters::creature)),
    ..oracle::BANKRUPT_IN_BLOOD
};
