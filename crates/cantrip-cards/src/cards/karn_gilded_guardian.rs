use cantrip_api::*;
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

/// "When Karn enters, draw a card for each color among other artifacts you control." A multicolored artifact counts once
/// for each of its colors (CR 105.2), and Karn itself is no other artifact.
fn draw_for_colors(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let others: Vec<ObjectId> = g
        .permanents_of(you)
        .into_iter()
        .filter(|&id| id != item.object && g.chars(id).is(CardType::Artifact))
        .collect();
    let colors = Color::ALL
        .iter()
        .filter(|&&color| others.iter().any(|&id| g.chars(id).colors.has(color)))
        .count();
    if colors > 0 {
        g.draw(you, colors as u32);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::THIS_ENTERS,
        draw_for_colors,
        "When Karn enters, draw a card for each color among other artifacts you control.",
    )],
    ..oracle::KARN_GILDED_GUARDIAN
};
