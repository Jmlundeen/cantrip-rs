use cantrip_api::*;
use cantrip_mechanics::choose_color;
use cantrip_mechanics::triggers::landfall;
use cantrip_oracle as oracle;

/// Landfall: add one mana of any color, chosen as the ability resolves (CR 106.1b). A triggered ability that adds mana
/// is not a mana ability (CR 605.1b): it goes on the stack, and it is independent of the Cobra once triggered.
fn add_mana_of_any_color(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let color = choose_color(g, item.controller, item.object)?;
    g.add_mana(item.controller, color.mana(), 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[AbilityDef::Triggered(TriggeredDef {
        text: "Landfall \u{2014} Whenever a land you control enters, add one mana of any color.",
        listens: EventMask::ZONE_CHANGE,
        matches: landfall,
        effect: add_mana_of_any_color,
        ..TriggeredDef::BLANK
    })],
    ..oracle::LOTUS_COBRA
};
