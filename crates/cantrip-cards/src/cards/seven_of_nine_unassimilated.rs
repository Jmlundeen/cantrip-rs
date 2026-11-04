use cantrip_api::*;
use cantrip_mechanics::{discard_chosen, lands, triggers::YOUR_FIRST_MAIN_PHASE};
use cantrip_oracle as oracle;

/// "At the beginning of your first main phase, draw a card. Then discard a card unless you control five or more
/// differently named lands" (CR 505.1a, 121.1, 701.9): the lands are counted after the draw and as the discard would
/// happen, and one of exactly five spares the discard.
fn draw_then_maybe_discard(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let player = item.controller;
    g.draw(player, 1);
    g.end_sentence();
    if lands::differently_named_lands(g, player) < 5 {
        discard_chosen(g, player, item.object, 1)?;
    }
    Ok(())
}

/// "{6}{U}: Seven of Nine's base power and toughness become 7/9" (CR 613.4b, 611.2a): a characteristic-setting effect
/// with no duration, so it lasts as long as the permanent stays the object it is.
fn become_seven_of_nine(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_effect(item.object, item.object, EffectKind::SetPt(7, 9), Duration::Indefinite);
    Ok(())
}

/// "Activate only if you control seven or more differently named lands" (CR 602.5): counted as the ability is activated.
fn seven_differently_named_lands(q: &dyn Query, asker: &Asker) -> bool {
    lands::differently_named_lands(q, asker.controller) >= 7
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            YOUR_FIRST_MAIN_PHASE,
            draw_then_maybe_discard,
            "At the beginning of your first main phase, draw a card. Then discard a card unless you control five or more differently named lands.",
        ),
        ability(
            cost("{6}{U}"),
            become_seven_of_nine,
            "{6}{U}: Seven of Nine's base power and toughness become 7/9. Activate only if you control seven or more differently named lands.",
        )
        .condition(seven_differently_named_lands),
    ],
    ..oracle::SEVEN_OF_NINE_UNASSIMILATED
};
