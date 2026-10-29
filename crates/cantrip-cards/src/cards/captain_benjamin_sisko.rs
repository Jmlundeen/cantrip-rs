use cantrip_api::*;
use cantrip_mechanics::{choose, tokens, top_of_library, triggers};
use cantrip_oracle as oracle;

fn make_token(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let player = item.controller;
    let dilemmas = g.history()
        .dilemmas_faced_this_turn(player);
    tokens::create(g, item.controller, &tokens::OFFICER, dilemmas)?;
    Ok(())
}

fn exile_and_cast(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let player = item.controller;
    let officers = g.state().battlefield()
        .iter()
        .filter(|&&id| g.controller(id) == player && g.chars(id).has_subtype(Subtype::Officer))
        .count() as u32;
    let cards = top_of_library(g, player, 5);
    let castable: Vec<ObjectId> = g.exile_all(&cards).into_iter().filter(|&card| g.obj(card).zone.is(ZoneKind::Exile) && g.mana_value(card) <= officers).collect();
    choose::cast_any_number_free(g, player, item.object, &castable)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(triggers::YOUR_END_STEP, make_token, "At the beginning of your end step, create a 1/1 red Officer creature token for each dilemma you faced this turn."),
        ability(cost("{2}{W}{U}{B}{R}{G}").and(TAP), exile_and_cast, "{2}{W}{U}{B}{R}{G}, {T}: Exile the top five cards of your library. You may cast any number of spells with mana value less than or equal to the number of Officers you control from among those cards without paying their mana costs.")
    ],
    ..oracle::CAPTAIN_BENJAMIN_SISKO
};

pub static EXTRA: &[&CardDef] = &[&tokens::OFFICER];
