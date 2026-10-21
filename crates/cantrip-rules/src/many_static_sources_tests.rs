//! Every permanent's static abilities apply, however many permanents have them (CR 604.1, 613.1): the layers once looked at
//! only the first 32 sources on the battlefield, so a lord or an anthem after a board of tokens or Equipment with static
//! abilities did nothing. Test cards: no printed card is needed to fill a board.

use crate::{Game, Setup};
use cantrip_api::*;

const YOU: PlayerId = PlayerId(0);

fn nothing(_: &dyn LayerQuery, _: ObjectId, _: ObjectId, _: &Chars) -> bool {
    false
}

fn unchanged(_: &dyn LayerQuery, _: ObjectId, _: &mut Chars) {}

/// A permanent with a static ability that affects nothing: one more static source on the battlefield.
static IDOL: CardDef = CardDef {
    name: "Static Test Idol",
    types: TypeSet::of(&[CardType::Artifact]),
    statics: &[static_ability(Layer::PtModify, nothing, unchanged, "Nothing is affected.")],
    ..CardDef::BLANK
};

fn your_creatures(view: &dyn LayerQuery, source: ObjectId, affected: ObjectId, chars: &Chars) -> bool {
    chars.is(CardType::Creature) && view.state().obj(affected).controller == view.state().obj(source).controller
}

fn plus_one(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.power = chars.power.map(|p| p + 1);
}

/// "Creatures you control get +1/+0", in layer 7c (CR 613.4c).
static BANNER: CardDef = CardDef {
    name: "Static Test Banner",
    types: TypeSet::of(&[CardType::Enchantment]),
    statics: &[static_ability(Layer::PtModify, your_creatures, plus_one, "Creatures you control get +1/+0.")],
    ..CardDef::BLANK
};

static BEAR: CardDef = CardDef {
    name: "Static Test Bear",
    types: TypeSet::of(&[CardType::Creature]),
    power: Some(2),
    toughness: Some(2),
    ..CardDef::BLANK
};

fn game() -> Game {
    let table: &'static CardTable = Box::leak(Box::new(CardTable::new([&IDOL, &BANNER, &BEAR])));
    Game::new(table, &Setup { decks: vec![Vec::new(), Vec::new()], ..Default::default() })
}

fn put(game: &mut Game, card: &'static CardDef) -> ObjectId {
    let mut obj = Obj::new(game.cards.id_of(card), YOU, Zone::BATTLEFIELD);
    obj.timestamp = game.state.timestamp();
    let id = game.state.push_obj(obj);
    game.state.zone_push(Zone::BATTLEFIELD, id);
    // As a move onto the battlefield does (`ops::note_statics`): the layers look for static sources once any has arrived.
    if !card.statics.is_empty() {
        game.state.see_statics();
    }
    id
}

/// Forty idols and then the banner: the banner is the 41st static source on the battlefield, and the bear still gets +1/+0.
#[test]
fn a_static_ability_applies_after_forty_other_static_sources() {
    let mut game = game();
    let bear = put(&mut game, &BEAR);
    for _ in 0..40 {
        put(&mut game, &IDOL);
    }
    assert_eq!(game.chars(bear).power, Some(2), "the idols affect nothing");
    put(&mut game, &BANNER);
    assert_eq!(game.chars(bear).power, Some(3), "the 41st static source applies (CR 604.1, 613.4c)");
}
