use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{library, select, targets, tokens};
use cantrip_oracle as oracle;

/// "Look at the top six cards of your library. You may put a land card from among them onto the battlefield tapped. Put the
/// rest on the bottom of your library in a random order."
fn look_for_a_land(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    library::look_at_top_onto_battlefield_select(g, item.controller, item.object, 6, select::LAND_CARD, true, library::Rest::BottomRandom)?;
    Ok(())
}

/// "Exile target card from a graveyard. If a creature card was exiled this way, create a Food token" (CR 701.13, 111.10b):
/// nothing at all if the target is no longer legal (CR 608.2b), and a Food only if the card really was exiled and was a
/// creature card.
fn exile_and_make_food(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(card) = targets::object(g, item, 0) else { return Ok(()) };
    let creature = g.chars(card).is(CardType::Creature);
    if !g.exile_all(&[card]).is_empty() && creature {
        tokens::create_food(g, item.controller)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            THIS_ENTERS,
            look_for_a_land,
            "When Ignis Scientia enters, look at the top six cards of your library. You may put a land card from among them onto the battlefield tapped. Put the rest on the bottom of your library in a random order.",
        ),
        ability(
            cost("{1}{G}{U}").and(TAP),
            exile_and_make_food,
            "I've Come Up with a New Recipe! \u{2014} {1}{G}{U}, {T}: Exile target card from a graveyard. If a creature card was exiled this way, create a Food token.",
        )
        .targets(&[targets::CARD_IN_A_GRAVEYARD]),
    ],
    ..oracle::IGNIS_SCIENTIA
};

pub static EXTRA: &[&CardDef] = &[&tokens::FOOD];
