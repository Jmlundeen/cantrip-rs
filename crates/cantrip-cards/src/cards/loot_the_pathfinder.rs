use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "Exhaust — {U}, {T}: Draw three cards."
fn draw_three(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.draw(item.controller, 3);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(cost("{G}, {T}"), add_any_color(3), "Exhaust \u{2014} {G}, {T}: Add three mana of any one color. (Activate each exhaust ability only once.)")
            .exhaust(),
        ability(cost("{U}, {T}"), draw_three, "Exhaust \u{2014} {U}, {T}: Draw three cards.")
            .exhaust(),
        ability(cost("{R}, {T}"), effects::damage_first_target::<3>, "Exhaust \u{2014} {R}, {T}: Loot, the Pathfinder deals 3 damage to any target.")
            .targets(&[targets::ANY])
            .exhaust(),
    ],
    ..oracle::LOOT_THE_PATHFINDER
};
