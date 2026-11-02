use cantrip_api::*;
use cantrip_mechanics::{damage_target, library, targets, tokens};
use cantrip_oracle as oracle;

/// "Creative Outburst deals 5 damage to any target. Look at the top five cards of your library. Put one of them into your hand
/// and the rest on the bottom of your library in a random order." (CR 608.2, 120.3).
fn outburst(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 5);
    g.end_sentence();
    library::look_at_top(
        g,
        item.controller,
        item.object,
        5,
        (1, 1),
        false,
        |_| true,
        library::Rest::BottomRandom,
    )?;
    Ok(())
}

/// "{U/R}{U/R}, Discard this card: Create a Treasure token." (CR 111.10a).
fn create_treasure_token(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create_treasure(g, item.controller)
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::ANY], outburst),
    abilities: &[
        ability(
            cost("{U/R}{U/R}").and(DISCARD_THIS),
            create_treasure_token,
            "{U/R}{U/R}, Discard this card: Create a Treasure token.",
        )
        .from(ZoneKind::Hand),
    ],
    ..oracle::CREATIVE_OUTBURST
};

pub static EXTRA: &[&CardDef] = &[&tokens::TREASURE];
