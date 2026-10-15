use cantrip_api::*;
use cantrip_mechanics::suspend;
use cantrip_oracle as oracle;

fn three_of_one_color(g: &mut dyn Game, asker: &Asker, color: Option<Color>) -> Result<(), Suspend> {
    g.add_mana(asker.controller, color.unwrap_or(Color::White).mana(), 3);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Activated(ActivatedDef {
            text: "{T}, Sacrifice this artifact: Add three mana of any one color.",
            cost: Cost { tap: true, sacrifice_self: true, ..Cost::NONE },
            mana: Some(three_of_one_color),
            any_color: true,
            adds_any: 3,
            ..ActivatedDef::BLANK
        }),
        suspend::ability(ManaCost::ZERO, "Suspend 3\u{2014}{0}"),
    ],
    suspend: Some(SuspendDef { count: 3, cost: ManaCost::ZERO }),
    ..oracle::LOTUS_BLOOM
};
