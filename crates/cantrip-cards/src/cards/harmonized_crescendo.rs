use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Choose a creature type. Draw a card for each permanent you control of that type." The type is chosen as the spell
/// resolves (CR 608.2d) and the permanents counted after (CR 608.2c). Only a creature or a Kindred permanent has a
/// creature type (CR 205.3d). The draws are one instruction (CR 121.2a). (Convoke is a characteristic of the card:
/// `pay_with`.)
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let choices = Subtype::CREATURE_TYPES.to_vec();
    let Answer::Subtype(chosen) = g.ask(you, RequestKind::ChooseSubtype { source: item.object, choices })? else {
        return Ok(());
    };
    let count = g
        .permanents_of(you)
        .into_iter()
        .filter(|&id| {
            let c = g.chars(id);
            (c.is(CardType::Creature) || c.is(CardType::Kindred)) && c.has_subtype(chosen)
        })
        .count();
    if count > 0 {
        g.draw(you, count as u32);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], effect),
    ..oracle::HARMONIZED_CRESCENDO
};
