//! "Whenever [a player] activates an ability that isn't a mana ability" (CR 602.2, 605.1a, 603.2): the engine records a mana
//! ability's activation in the history but never emits it as an event (`EventMask::of`), so no trigger is ever asked about one
//! and the matchers' "isn't a mana ability" guards are not reached in play. They stay for the cards that trigger on mana abilities
//! too (Elrond, Moon-Reader; Ceaseless Searblades), and are judged here directly on the event a mana ability's activation would
//! be: Harsh Mentor's matcher (`triggers::opponent_activates_ability_of_artifact_creature_or_land`) and Burning-Tree Shaman's,
//! each refusing an opponent's Mox Ruby's {T}: Add {R} and taking Ghost Vacuum's {T}: Exile target card.

use cantrip_engine::*;

const YOU: PlayerId = PlayerId(0);
const THEM: PlayerId = PlayerId(1);

/// A game where Bot2 controls a Mox Ruby and a Ghost Vacuum, and Bot1 a Burning-Tree Shaman. The game and those three.
fn board() -> (Game, ObjectId, ObjectId, ObjectId) {
    let island = card("Island").unwrap();
    let mut game = Game::new(cards(), &Setup { decks: vec![vec![island; 30], vec![island; 30]], seed: 1, shuffle: false, opening_chooser: None, ..Default::default() });
    game.run();
    let mut put = |name: &str, controller: PlayerId| {
        let id = game.state.push_obj(Obj::new(card(name).unwrap(), controller, Zone::BATTLEFIELD));
        game.state.zone_push(Zone::BATTLEFIELD, id);
        id
    };
    let mox = put("Mox Ruby", THEM);
    let vacuum = put("Ghost Vacuum", THEM);
    let shaman = put("Burning-Tree Shaman", YOU);
    (game, mox, vacuum, shaman)
}

fn activated(game: &Game, source: ObjectId) -> Event {
    Event::AbilityActivated { source, ability: AbilityRef { card: game.state.obj(source).card, index: 0 }, player: THEM, by: Grant::NONE }
}

#[test]
fn harsh_mentors_trigger_is_not_for_a_mana_ability() {
    let (game, mox, vacuum, shaman) = board();
    let me = Asker { source: shaman, controller: YOU };
    let matches = cantrip_mechanics::triggers::opponent_activates_ability_of_artifact_creature_or_land;
    assert!(matches(&game, &me, &activated(&game, mox)).is_none(), "a mana ability triggers it");
    assert!(matches(&game, &me, &activated(&game, vacuum)).is_some(), "an artifact's other ability does not");
}

#[test]
fn burning_tree_shamans_trigger_is_not_for_a_mana_ability() {
    let (game, mox, vacuum, shaman) = board();
    let AbilityDef::Triggered(trigger) = game.cards.ability(AbilityRef { card: game.state.obj(shaman).card, index: 0 }) else {
        panic!("Burning-Tree Shaman's first ability is its trigger")
    };
    assert!((trigger.matches)(&game, shaman, &activated(&game, mox)).is_none(), "a mana ability triggers it");
    assert!((trigger.matches)(&game, shaman, &activated(&game, vacuum)).is_some(), "a nonmana ability does not");
}
