use cantrip_api::*;
use cantrip_mechanics::{lands, lands_you_control, pump, targets, trample_until_end_of_turn};
use cantrip_oracle as oracle;

/// "I — Mill five cards." (CR 701.17a)
fn mill_five(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.mill(item.controller, 5);
    Ok(())
}

/// "II — Return all land cards from your graveyard to the battlefield tapped." (CR 603.6a, 110.5b)
fn return_lands(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    lands::return_all_land_cards_from_graveyard_tapped(g, item.controller)
}

/// "III — Until end of turn, another target creature you control gains trample and gets +X/+X, where X is the number of lands you
/// control." (CR 611.2c, 608.2h): X is the lands its controller controls as the chapter resolves, counted once, not again as
/// the lands change; nothing happens if the target is no longer legal (CR 608.2b).
fn trample_and_grow(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    let x = lands_you_control(g, item.controller) as i32;
    instruction(g, |g| {
        trample_until_end_of_turn(g, item.object, creature);
        pump(g, item.object, creature, x, x);
    });
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        chapter(1, mill_five, "I \u{2014} Mill five cards."),
        chapter(2, return_lands, "II \u{2014} Return all land cards from your graveyard to the battlefield tapped."),
        chapter(
            3,
            trample_and_grow,
            "III \u{2014} Until end of turn, another target creature you control gains trample and gets +X/+X, where X is the number of lands you control.",
        )
        .targets(&[targets::OTHER_CREATURE_YOU_CONTROL]),
    ],
    ..oracle::SUMMON_TITAN
};
