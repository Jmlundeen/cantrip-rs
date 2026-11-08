use cantrip_api::*;
use cantrip_mechanics::triggers::YOU_DRAW_YOUR_SECOND_CARD;
use cantrip_mechanics::tokens;
use cantrip_oracle as oracle;

/// "Seek out — Whenever you draw your second card each turn, create two tapped Treasure tokens." (CR 121.2a: drawing
/// cards; CR 603.2a: the trigger fires once per turn, on the second draw, whoever's turn it is.) Both tokens enter
/// together, tapped, as one event (CR 111.10a).
fn create_two_tapped_treasures(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create_by_rule(g, item.controller, &tokens::TREASURE, 2, true)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        YOU_DRAW_YOUR_SECOND_CARD,
        create_two_tapped_treasures,
        "Seek out \u{2014} Whenever you draw your second card each turn, create two tapped Treasure tokens.",
    )],
    ..oracle::ROM_AND_LEETA_UNLIKELY_PAIR
};

pub static EXTRA: &[&CardDef] = &[&tokens::TREASURE];

