use cantrip_api::*;
use cantrip_mechanics::library::{self, Rest};
use cantrip_oracle as oracle;

/// "Look at the top three cards of your library. Put one of them into your hand and the rest on the bottom of your library in any
/// order" (CR 401.2, 401.4): one card is taken, whatever it is; a library of fewer cards shows what it has (CR 609.3).
fn look_at_three_take_one(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    library::look_at_top(g, item.controller, item.object, 3, (1, 1), false, |_| true, Rest::BottomChosen)?;
    Ok(())
}

/// Flying, vigilance and lifelink are core keywords.
pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        cost("{U}{B}").and(EXILE_THIS),
        look_at_three_take_one,
        "{U}{B}, Exile this card from your graveyard: Look at the top three cards of your library. Put one of them into your hand and the rest on the bottom of your library in any order.",
    )
    .from(ZoneKind::Graveyard)],
    ..oracle::MORBIUS_THE_LIVING_VAMPIRE
};
