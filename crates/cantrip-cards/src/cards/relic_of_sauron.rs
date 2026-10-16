use cantrip_api::*;
use cantrip_mechanics::discard_chosen;
use cantrip_oracle as oracle;

/// "Add two mana in any combination of {U}, {B}, and/or {R}" (CR 106.1b, 605.3): each mana a color of its own choosing,
/// chosen as the mana is added.
fn add_two(g: &mut dyn Game, asker: &Asker, _: Option<Color>) -> Result<(), Suspend> {
    let choices = ColorSet(ColorSet::of(Color::Blue).0 | ColorSet::of(Color::Black).0 | ColorSet::of(Color::Red).0);
    if let Answer::Colors(colors) = g.ask(asker.controller, RequestKind::ChooseColors { source: asker.source, choices, count: 2 })? {
        for color in colors {
            g.add_mana(asker.controller, color.mana(), 1);
        }
    }
    Ok(())
}

/// Draw two cards, then discard a card.
fn loot(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.draw(item.controller, 2);
    discard_chosen(g, item.controller, item.object, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add_by(add_two), "{T}: Add 2 mana in any combination of {U}, {B}, and/or {R}."),
        ability(cost("{3}, {T}"), loot, "{3}, {T}: Draw two cards, then discard a card."),
    ],
    ..oracle::RELIC_OF_SAURON
};
