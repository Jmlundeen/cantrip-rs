//! What can be cast counts a mana ability without {T} as many times as its cost can be paid (CR 602.2, 605): Ashnod's Altar
//! with four creatures makes {C}{C} four times, and Myr Battlesphere ({7}) is offered; with three it is not. A creature two
//! altars could sacrifice is sacrificed to one of them only.

use std::sync::OnceLock;

use cantrip_engine::*;

static BEAR: CardDef = CardDef { name: "Probe Bear", types: TypeSet::of(&[CardType::Creature]), power: Some(2), toughness: Some(2), ..CardDef::BLANK };

fn table() -> &'static CardTable {
    static TABLE: OnceLock<CardTable> = OnceLock::new();
    TABLE.get_or_init(|| CardTable::new(cantrip_cards::all().into_iter().chain([&BEAR])))
}

/// Whether, with `permanents` on Bot1's battlefield and `card` in hand, `card` is offered at Bot1's first main phase.
fn offered(permanents: &[&str], card: &str) -> bool {
    let find = |name: &str| table().find(name).unwrap_or_else(|| panic!("no card {name}"));
    let mut deck = vec![find(card)];
    deck.extend(vec![find("Island"); 29]);
    let mut game = Game::new(table(), &Setup { decks: vec![deck, vec![find("Island"); 30]], seed: 1, shuffle: false, opening_chooser: None, ..Default::default() });
    for name in permanents {
        let id = game.state.push_obj(Obj::new(find(name), PlayerId(0), Zone::BATTLEFIELD));
        game.state.zone_push(Zone::BATTLEFIELD, id);
    }
    for _ in 0..50 {
        game.run();
        let request = game.pending().cloned().expect("a question");
        if let RequestKind::Priority { options } = &request.kind
            && request.player == PlayerId(0)
            && game.state.turn.step() == Step::PrecombatMain
        {
            return options.iter().any(|o| o.kind == ActionKind::CastSpell && o.source.is_some_and(|c| game.name(c) == card));
        }
        game.answer(request.id, request.player, Answer::Pass).unwrap();
    }
    panic!("no main phase");
}

#[test]
fn ashnods_altar_with_four_creatures_offers_myr_battlesphere() {
    assert!(offered(&["Ashnod's Altar", "Probe Bear", "Probe Bear", "Probe Bear", "Probe Bear"], "Myr Battlesphere"));
    assert!(!offered(&["Ashnod's Altar", "Probe Bear", "Probe Bear", "Probe Bear"], "Myr Battlesphere"));
}

/// Two altars, three creatures: each creature is sacrificed once, to one altar ({C}{C} or one mana of any color), so at
/// most six mana - {7} is not offered, though counting each altar once per creature would make twelve.
#[test]
fn a_creature_two_altars_could_sacrifice_is_counted_once() {
    assert!(!offered(&["Ashnod's Altar", "Phyrexian Altar", "Probe Bear", "Probe Bear", "Probe Bear"], "Myr Battlesphere"));
    assert!(offered(&["Ashnod's Altar", "Phyrexian Altar", "Probe Bear", "Probe Bear", "Probe Bear", "Probe Bear"], "Myr Battlesphere"));
}
