use cantrip_api::*;
use cantrip_mechanics::suspend;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP.and(SACRIFICE_THIS), add_any_color(3), "{T}, Sacrifice this artifact: Add three mana of any one color."),
        suspend::ability(ManaCost::ZERO, "Suspend 3\u{2014}{0}"),
    ],
    suspend: Some(SuspendDef { count: 3, cost: ManaCost::ZERO }),
    ..oracle::LOTUS_BLOOM
};
