use cantrip_api::*;
use cantrip_mechanics::{each_opponent_sacrifices, effects, targets, tokens};
use cantrip_oracle as oracle;

/// I: "Each opponent sacrifices a nontoken creature of their choice" (CR 701.21a, 101.4): every opponent chooses on the board
/// as it is before any of them is sacrificed, and the chosen creatures die together. An opponent with no nontoken creature
/// sacrifices nothing, a creature token not being one.
fn chapter_one(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    each_opponent_sacrifices(g, item.controller, item.object, 1, |q, id| {
        q.chars(id).is(CardType::Creature) && !q.obj(id).token
    })
}

/// II: "For each opponent, you create a 2/1 red Klingon Warrior creature token with haste" (CR 608.2c): one token for every
/// opponent still in the game, made together as one event. None for no opponents.
fn chapter_two(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let opponents = g.state().opponents(item.controller).count() as u32;
    tokens::create_klingons(g, item.controller, opponents)
}

/// III: "Exile target player's graveyard. Each opponent loses life equal to the number of creature cards exiled this way"
/// (CR 406.2, 119.3, 608.2c). The graveyard's cards are counted as they lie there, before the exile: the shared exile takes
/// every one of them. Nothing if the target player is no longer legal (CR 608.2b).
fn chapter_three(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(player) = targets::player(g, item, 0) else { return Ok(()) };
    let creatures = g
        .state()
        .zone(Zone::graveyard(player))
        .iter()
        .filter(|&&card| g.chars(card).is(CardType::Creature))
        .count() as i32;
    effects::exile_first_target_players_graveyard(g, item)?;
    g.each_opponent_loses_life(item.controller, creatures);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        chapter(1, chapter_one, "I \u{2014} Each opponent sacrifices a nontoken creature of their choice."),
        chapter(
            2,
            chapter_two,
            "II \u{2014} For each opponent, you create a 2/1 red Klingon Warrior creature token with haste.",
        ),
        chapter(
            3,
            chapter_three,
            "III \u{2014} Exile target player's graveyard. Each opponent loses life equal to the number of creature cards exiled this way.",
        )
        .targets(&[targets::PLAYER]),
    ],
    ..oracle::SINS_OF_THE_FATHER
};

pub static EXTRA: &[&CardDef] = &[&tokens::KLINGON];
