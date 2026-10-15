use cantrip_api::*;
use cantrip_mechanics::{damage_target, targets};
use cantrip_oracle as oracle;

/// "Exhaust — {G}, {T}: Add three mana of any one color." A mana ability, so listed first.
fn three_of_one_color(g: &mut dyn Game, asker: &Asker, color: Option<Color>) -> Result<(), Suspend> {
    g.add_mana(asker.controller, color.unwrap_or(Color::White).mana(), 3);
    Ok(())
}

/// "Exhaust — {U}, {T}: Draw three cards."
fn draw_three(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.draw(item.controller, 3);
    Ok(())
}

/// "Exhaust — {R}, {T}: Loot, the Pathfinder deals 3 damage to any target."
fn three_damage(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 3);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Activated(ActivatedDef {
            text: "Exhaust \u{2014} {G}, {T}: Add three mana of any one color. (Activate each exhaust ability only once.)",
            cost: Cost { mana: mana(0, 0, 0, 0, 0, 1), tap: true, ..Cost::NONE },
            mana: Some(three_of_one_color),
            any_color: true,
            adds_any: 3,
            exhaust: true,
            ..ActivatedDef::BLANK
        }),
        AbilityDef::Activated(ActivatedDef {
            text: "Exhaust \u{2014} {U}, {T}: Draw three cards.",
            cost: Cost { mana: mana(0, 0, 1, 0, 0, 0), tap: true, ..Cost::NONE },
            effect: draw_three,
            exhaust: true,
            ..ActivatedDef::BLANK
        }),
        AbilityDef::Activated(ActivatedDef {
            text: "Exhaust \u{2014} {R}, {T}: Loot, the Pathfinder deals 3 damage to any target.",
            cost: Cost { mana: mana(0, 0, 0, 0, 1, 0), tap: true, ..Cost::NONE },
            targets: &[targets::ANY],
            effect: three_damage,
            exhaust: true,
            ..ActivatedDef::BLANK
        }),
    ],
    ..oracle::LOOT_THE_PATHFINDER
};
