use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

/// "I, II, III — Each player draws a card. This Saga deals 2 damage to each player." (CR 121.1, 120.2, 101.4) Every
/// player in the game draws in turn order first, then the damage is dealt to all of them at once, one event each; a
/// player who has left the game is dealt neither (CR 800.4a).
fn each_player_draws_then_takes_two(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::each_player_draws_a_card(g, item)?;
    g.end_sentence();
    let hits: Vec<(ObjectId, Target, u32)> = g.state().apnap().map(|p| (item.object, Target::Player(p), 2)).collect();
    g.damage_together(&hits);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        chapter(
            1,
            each_player_draws_then_takes_two,
            "I — Each player draws a card. This Saga deals 2 damage to each player.",
        ),
        chapter(
            2,
            each_player_draws_then_takes_two,
            "II — Each player draws a card. This Saga deals 2 damage to each player.",
        ),
        chapter(
            3,
            each_player_draws_then_takes_two,
            "III — Each player draws a card. This Saga deals 2 damage to each player.",
        ),
    ],
    ..oracle::YEAR_OF_HELL
};
