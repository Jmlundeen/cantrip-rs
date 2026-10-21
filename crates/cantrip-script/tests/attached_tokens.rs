//! "Create a [token] attached to [a permanent]" for an Equipment token (CR 301.5e): an Equipment token that could not equip
//! the permanent is created unattached, where an Aura token that could not enchant it is not created at all (CR 303.4i,
//! shown by Monstrous Rage under Giver of Runes' protection from colorless). No printed card in the engine creates an
//! Equipment token attached to something, so the path is played here on a table of test-only cards.

use std::sync::OnceLock;

use cantrip_api::Game as Rules;
use cantrip_engine::*;

fn permanent(q: &dyn Query, _: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if q.obj(id).zone.is(ZoneKind::Battlefield))
}

/// A colorless Equipment token, to be created attached.
static GEAR: CardDef = CardDef {
    name: "Probe Gear",
    types: TypeSet::of(&[CardType::Artifact]),
    subtypes: SubtypeSet::of(&[Subtype::Equipment]),
    ..CardDef::BLANK
};

/// Create a Probe Gear token attached to target permanent.
fn arm(g: &mut dyn Rules, item: &StackItem) -> Result<(), Suspend> {
    if let (Target::Object(host), 1) = (item.target(0), item.target_count) {
        let gear = g.card_id(&GEAR);
        g.create_token_attached(gear, item.controller, host)?;
    }
    Ok(())
}

static ONE: [TargetSpec; 1] = [target(permanent)];

static ARM: CardDef = CardDef { name: "Probe Arm", types: TypeSet::of(&[CardType::Instant]), spell: spell(&ONE, arm), ..CardDef::BLANK };

static BRUTE: CardDef =
    CardDef { name: "Probe Brute", types: TypeSet::of(&[CardType::Creature]), power: Some(2), toughness: Some(2), ..CardDef::BLANK };

static FILLER: CardDef = CardDef { name: "Filler", has_cost: false, types: TypeSet::of(&[CardType::Land]), ..CardDef::BLANK };

fn table() -> &'static CardTable {
    static TABLE: OnceLock<CardTable> = OnceLock::new();
    TABLE.get_or_init(|| CardTable::new([&ARM, &GEAR, &BRUTE, &FILLER]))
}

/// The first seat holds a Probe Arm, and has a Probe Brute and a Filler on the battlefield.
fn deal() -> (Game, ObjectId, ObjectId) {
    let filler = table().find("Filler").unwrap();
    let mut deck = vec![table().find("Probe Arm").unwrap()];
    deck.extend(std::iter::repeat(filler).take(29));
    let mut game = Game::new(table(), &Setup { decks: vec![deck, vec![filler; 30]], seed: 1, shuffle: false, ..Setup::default() });
    let mut put = |name: &str| {
        let id = game.state.push_obj(Obj::new(table().find(name).unwrap(), PlayerId(0), Zone::BATTLEFIELD));
        game.state.zone_push(Zone::BATTLEFIELD, id);
        id
    };
    let (brute, land) = (put("Probe Brute"), put("Filler"));
    game.verify = true;
    game.run();
    (game, brute, land)
}

/// The first seat casts Probe Arm at `host`, and it resolves.
fn arm_at(game: &mut Game, host: ObjectId) {
    let mut cast = false;
    for _ in 0..100 {
        game.run();
        if cast && game.state.stack.is_empty() {
            return;
        }
        let Some(request) = game.pending().cloned() else { panic!("the game ended") };
        let answer = match &request.kind {
            RequestKind::Priority { options } if request.player == PlayerId(0) && !cast => {
                match options.iter().find(|o| o.kind == ActionKind::CastSpell) {
                    Some(option) => {
                        cast = true;
                        // A lone target is chosen with the action (its candidates are on the option).
                        let mut action = Action::of(option);
                        action.targets = vec![Target::Object(host)];
                        Answer::Act(action)
                    }
                    None => Answer::Pass,
                }
            }
            RequestKind::Priority { .. } => Answer::Pass,
            RequestKind::ChooseTarget { .. } => Answer::Targets(vec![Target::Object(host)]),
            other => panic!("unexpected question {other:?}"),
        };
        game.answer(request.id, request.player, answer).unwrap();
    }
    panic!("Probe Arm did not resolve");
}

/// The Probe Gear tokens on the battlefield, with what each is attached to.
fn gear(game: &Game) -> Vec<Option<ObjectId>> {
    game.state.battlefield().iter().filter(|&&id| game.name(id) == "Probe Gear").map(|&id| game.state.obj(id).attached_to).collect()
}

/// CR 301.5e: attached to a creature, which it could equip, the Equipment token enters attached to it.
#[test]
fn an_equipment_token_enters_attached_to_a_creature() {
    let (mut game, brute, _) = deal();
    arm_at(&mut game, brute);
    assert_eq!(gear(&game), vec![Some(brute)]);
}

/// CR 301.5e: "attached to" a land, which an Equipment can't equip, the token is still created - unattached.
#[test]
fn an_equipment_token_that_could_not_equip_its_host_is_created_unattached() {
    let (mut game, _, land) = deal();
    arm_at(&mut game, land);
    assert_eq!(gear(&game), vec![None]);
}
