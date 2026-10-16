use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{cards_in_hand, creatures_you_control, effects, lands_you_control, tokens};
use cantrip_oracle as oracle;

/// Whether some opponent of `you` has more of `count` than you (CR 102.2). Each comparison is made as it comes in the
/// text, as the ability resolves: nothing is remembered from an earlier one.
fn an_opponent_has_more(g: &dyn Game, you: PlayerId, count: fn(&dyn Query, PlayerId) -> i64) -> bool {
    let mine = count(g, you);
    g.state().opponents(you).any(|opponent| count(g, opponent) > mine)
}

fn life(q: &dyn Query, player: PlayerId) -> i64 {
    q.state().player(player).life as i64
}

/// "When Beza enters, create a Treasure token if an opponent controls more lands than you. You gain 4 life if an
/// opponent has more life than you. Create two 1/1 blue Fish creature tokens if an opponent controls more creatures than
/// you. Draw a card if an opponent has more cards in hand than you." The four comparisons are made one after another
/// as the ability resolves (CR 608.2c): a Treasure that is a creature counts for the third.
fn enters(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    if an_opponent_has_more(g, you, |q, p| lands_you_control(q, p) as i64) {
        tokens::create_treasure(g, you)?;
    }
    if an_opponent_has_more(g, you, life) {
        effects::gain_life(g, item, 4)?;
    }
    if an_opponent_has_more(g, you, |q, p| creatures_you_control(q, p) as i64) {
        tokens::create(g, you, &tokens::FISH, 2)?;
    }
    if an_opponent_has_more(g, you, |q, p| cards_in_hand(q, p) as i64) {
        effects::draw(g, item, 1)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, enters, "When Beza enters, create a Treasure token if an opponent controls more lands than you. You gain 4 life if an opponent has more life than you. Create two 1/1 blue Fish creature tokens if an opponent controls more creatures than you. Draw a card if an opponent has more cards in hand than you.")],
    ..oracle::BEZA_THE_BOUNDING_SPRING
};

pub static EXTRA: &[&CardDef] = &[&tokens::TREASURE, &tokens::FISH];
