use cantrip_api::*;
use cantrip_mechanics::{
    effects::{ask_may, players_discard_hands_then_draw_seven},
    triggers::PROWESS,
};
use cantrip_oracle as oracle;

const WHEEL: Question = Question::new(
    "DiscardHandToDrawSeven",
    "discard your hand and draw seven cards?",
    "You may discard even an empty hand",
    "Discard hand, draw seven",
    "Keep hand",
).no_default();

/// "Each player may discard their hand and draw seven cards." Choices are made in APNAP order before
/// simultaneous discards (CR 101.4, 701.9a); consenting players then draw in APNAP order (CR 121.2c).
fn arc_of_fortune(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let players: Vec<PlayerId> = g.state().apnap().collect();
    let mut willing = Vec::new();
    for player in players {
        // CR 121.2b: a player who can't draw seven cards is not offered the choice to.
        if !cant_draw(g, player, 7) && ask_may(g, player, item.object, Purpose::Ask(&WHEEL))? {
            willing.push(player);
        }
    }
    players_discard_hands_then_draw_seven(g, &willing);
    Ok(())
}

/// Variable Chaser's prepare spell (CR 722.3c), a copy cast from exile.
pub static ARC_OF_FORTUNE: CardDef = CardDef {
    name: "Arc of Fortune",
    cost: mana("{2}{U}"),
    types: TypeSet::of(&[CardType::Sorcery]),
    spell: spell(&[], arc_of_fortune),
    text: "Each player may discard their hand and draw seven cards.",
    ..CardDef::BLANK
};

pub static CARD: CardDef = CardDef {
    abilities: &[PROWESS],
    enters: Some(cantrip_mechanics::prepare::enters_prepared),
    prepare: Some(&ARC_OF_FORTUNE),
    ..oracle::VARIABLE_CHASER
};

pub static EXTRA: &[&CardDef] = &[&ARC_OF_FORTUNE];
