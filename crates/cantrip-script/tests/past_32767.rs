//! Power and toughness past 32,767, which the rules set no limit on (CR 107.1, 613.4; #2336): a base set in layer 7b, a pump
//! in 7c and an effect that adds a creature's own power to itself are summed exactly, and a power driven far below zero keeps
//! its value. No printed card reaches such sizes on today's boards through a short game, so the effects are put on directly.

use std::sync::OnceLock;

use cantrip_engine::*;

fn itself(_: &dyn LayerQuery, source: ObjectId, id: ObjectId, _: &Chars) -> bool {
    source == id
}

fn plus_its_power(view: &dyn LayerQuery, source: ObjectId, c: &mut Chars) {
    let power = view.chars(source).power.unwrap_or(0);
    c.power = c.power.map(|p| p + power);
}

/// "This creature gets +X/+0, where X is its power."
static DOUBLER: CardDef = CardDef {
    name: "Probe Doubler",
    types: TypeSet::of(&[CardType::Creature]),
    power: Some(2),
    toughness: Some(2),
    statics: &[static_ability(Layer::PtModify, itself, plus_its_power, "This creature gets +X/+0, where X is its power.")],
    ..CardDef::BLANK
};

fn table() -> &'static CardTable {
    static TABLE: OnceLock<CardTable> = OnceLock::new();
    TABLE.get_or_init(|| CardTable::new(cantrip_cards::all().into_iter().chain([&DOUBLER])))
}

fn creature() -> (Game, ObjectId) {
    let filler = table().find("Plains").unwrap();
    let mut game = Game::new(table(), &Setup { decks: vec![vec![filler; 20], vec![filler; 20]], seed: 1, shuffle: false, ..Setup::default() });
    let mut obj = Obj::new(table().find("Probe Doubler").unwrap(), PlayerId(0), Zone::BATTLEFIELD);
    obj.timestamp = 10;
    let id = game.state.push_obj(obj);
    game.state.zone_push(Zone::BATTLEFIELD, id);
    game.state.see_statics();
    (game, id)
}

fn effect(game: &mut Game, id: ObjectId, timestamp: u32, kind: EffectKind) {
    game.state.effect_push(ContinuousEffect {
        source: id,
        controller: PlayerId(0),
        timestamp,
        affected: id,
        kind,
        duration: Duration::EndOfTurn,
        words: Words::NONE,
    });
}

/// A base of 30,000/30,000 (7b), +5,000/+5,000 (7c) and its own power added again (7c, the Doubler) is 70,000/35,000.
#[test]
fn a_base_a_pump_and_a_doubling_past_32767_are_exact() {
    let (mut game, id) = creature();
    effect(&mut game, id, 20, EffectKind::SetPt(30_000, 30_000));
    effect(&mut game, id, 30, EffectKind::Pump(5_000, 5_000));
    let c = game.chars(id);
    assert_eq!((c.power, c.toughness), (Some(70_000), Some(35_000)));
}

/// A power pushed to -100,000 stays that, and the toughness beside it is untouched.
#[test]
fn a_power_far_below_zero_keeps_its_value() {
    let (mut game, id) = creature();
    effect(&mut game, id, 20, EffectKind::Pump(-100_000, 0));
    let c = game.chars(id);
    assert_eq!((c.power, c.toughness), (Some(-199_996), Some(2)));
}
