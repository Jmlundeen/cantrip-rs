//! Dr. Beverly Crusher duplicates life-gain triggers of permanents its controller controls (CR 603.2d). The printed
//! pool has no opposing permanent that triggers when its opponent gains life, so a test-only permanent supplies that
//! event shape without adding an unrelated production card.

use crate::{Cx, Game, Setup};
use cantrip_api::*;

const YOU: PlayerId = PlayerId(0);
const OPPONENT: PlayerId = PlayerId(1);

fn any_player_gains_life(_: &dyn Query, _: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::LifeChange { delta, .. } if *delta > 0 => Fired::PLAIN,
        _ => None,
    }
}

fn no_op(_: &mut dyn cantrip_api::Game, _: &StackItem) -> Result<(), Suspend> {
    Ok(())
}

static LIFE_GAIN_PROBE: CardDef = CardDef {
    name: "Crusher Trigger Probe",
    types: TypeSet::of(&[CardType::Creature]),
    abilities: &[triggered(
        when(EventMask::LIFE_CHANGE, any_player_gains_life),
        no_op,
        "Whenever a player gains life, this test ability triggers.",
    )],
    ..CardDef::BLANK
};

#[test]
fn crusher_does_not_duplicate_an_opponents_permanent_triggered_by_your_life_gain() {
    let crusher = cantrip_cards::all()
        .into_iter()
        .find(|card| card.name == "Dr. Beverly Crusher")
        .expect("Crusher is implemented");
    let mut definitions = cantrip_cards::all();
    definitions.push(&LIFE_GAIN_PROBE);
    let table: &'static CardTable = Box::leak(Box::new(CardTable::new(definitions)));
    let mut game = Game::new(
        table,
        &Setup {
            decks: vec![Vec::new(), Vec::new()],
            ..Default::default()
        },
    );

    let crusher_id = game
        .state
        .push_obj(Obj::new(table.id_of(crusher), YOU, Zone::BATTLEFIELD));
    game.state.zone_push(Zone::BATTLEFIELD, crusher_id);
    let opposing_probe = game.state.push_obj(Obj::new(
        table.id_of(&LIFE_GAIN_PROBE),
        OPPONENT,
        Zone::BATTLEFIELD,
    ));
    game.state.zone_push(Zone::BATTLEFIELD, opposing_probe);
    let own_probe = game.state.push_obj(Obj::new(
        table.id_of(&LIFE_GAIN_PROBE),
        YOU,
        Zone::BATTLEFIELD,
    ));
    game.state.zone_push(Zone::BATTLEFIELD, own_probe);

    let cx = Cx {
        g: &mut game,
        frame: usize::MAX,
        cursor: 0,
        mana_source: None,
        entering: None,
        lineage: Vec::new(),
        asked_at: None,
        doer: None,
        producing: None,
    };
    let event = Event::LifeChange {
        player: YOU,
        delta: 3,
    };
    let chars = cx.chars(crusher_id);
    let crusher_asker = Asker {
        source: crusher_id,
        controller: YOU,
        by: cx.link_nth(crusher_id, &chars, 0),
        spell: false,
    };
    let AbilityDef::Triggered(probe_trigger) = LIFE_GAIN_PROBE.abilities[0] else {
        panic!("probe has a triggered ability")
    };
    assert!(
        (probe_trigger.matches)(&cx, opposing_probe, &event).is_some(),
        "the opponent's permanent triggers on your life gain"
    );

    let additional = crusher.additional_triggers[0];
    let opponent_trigger = Triggering {
        source: opposing_probe,
        controller: OPPONENT,
        def: &probe_trigger,
        permanent: true,
        looks_back: false,
    };
    let own_trigger = Triggering {
        source: own_probe,
        controller: YOU,
        def: &probe_trigger,
        permanent: true,
        looks_back: false,
    };
    assert!(
        !additional(&cx, &crusher_asker, &event, &opponent_trigger),
        "Crusher must not duplicate the opponent's permanent trigger"
    );
    assert!(
        additional(&cx, &crusher_asker, &event, &own_trigger),
        "Crusher still duplicates its controller's permanent trigger"
    );
}
