//! A cast is the card's own option (its face, its own cost) and what made it possible (`ActionOption::granted_by`, with
//! `paid_by` for an alternative cost another object gives): every permission that would let a card be cast is an option of
//! its own, and the cast is made by the one chosen, on its terms (CR 601.2b, 601.3, 118.9). The cards here are small
//! probes for the permission machinery, not card implementations. Taken, as rules, from the tests of pull request #147.

use std::sync::OnceLock;

use cantrip_api::*;
use cantrip_rules::{Game as RulesGame, Setup};
use cantrip_state::{Frame, FrameKind};

const ME: PlayerId = PlayerId(0);

static PROBE_SPELL: CardDef = CardDef {
    name: "Route Probe",
    oracle: true,
    cost: ManaCost { generic: 2, ..ManaCost::ZERO },
    has_cost: true,
    types: TypeSet::of(&[CardType::Instant]),
    spell: Some(SpellDef::MODAL),
    ..CardDef::BLANK
};

/// An alternative cost that names no zone (Force of Will's): open from wherever a permission has the card cast.
static HAND_WAY: WayDef = WayDef {
    text: "hand-named cost",
    cost: Cost::mana(ManaCost::ZERO),
    from: ZoneKind::Hand,
    only_from: false,
    exile_after: false,
    condition: None,
    instant_speed: true,
    enters_attacking: false,
    kind: WayKind::Other,
};

/// One that casts it from its owner's graveyard only.
static GRAVEYARD_WAY: WayDef = WayDef {
    text: "graveyard alternate way",
    cost: Cost::mana(ManaCost::ZERO),
    from: ZoneKind::Graveyard,
    only_from: true,
    exile_after: false,
    condition: None,
    instant_speed: true,
    enters_attacking: false,
    kind: WayKind::Other,
};

static WAY_SPELL: CardDef = CardDef { name: "Route Way Probe", ways: &[HAND_WAY, GRAVEYARD_WAY], ..PROBE_SPELL };

static INTRINSIC_GRAVEYARD_SPELL: CardDef =
    CardDef { name: "Route Intrinsic Graveyard Probe", keywords: Keywords::of(&[Keyword::CastFromYourGraveyard]), ..PROBE_SPELL };

static ADVENTURE_FACE: CardDef = CardDef {
    name: "Route Adventure Probe Adventure",
    cost: ManaCost { generic: 1, ..ManaCost::ZERO },
    types: TypeSet::of(&[CardType::Sorcery]),
    ..PROBE_SPELL
};

static ADVENTURE_PROBE: CardDef = CardDef { name: "Route Adventure Probe", adventure: Some(&ADVENTURE_FACE), ..PROBE_SPELL };

fn cast_opponents_graveyard_card(g: &mut dyn cantrip_api::Game, item: &StackItem) -> Result<(), Suspend> {
    let opponent = PlayerId(1 - item.controller.0);
    if let Some(&card) = g.state().zone(Zone::graveyard(opponent)).first() {
        g.cast_during_resolution(card, None)?;
    }
    Ok(())
}

static RESOLUTION_PROBE: CardDef = CardDef {
    name: "Route Resolution Probe",
    cost: ManaCost::ZERO,
    spell: Some(SpellDef { targets: &[], effect: cast_opponents_graveyard_card }),
    ..PROBE_SPELL
};

static FLASHBACK_WAY: WayDef = WayDef { text: "Flashback {0}", exile_after: true, ..GRAVEYARD_WAY };

static FLASHBACK_SPELL: CardDef = CardDef { name: "Route Resolution Flashback Probe", ways: &[FLASHBACK_WAY], ..PROBE_SPELL };

fn cast_own_graveyard_card(g: &mut dyn cantrip_api::Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(&card) = g.state().zone(Zone::graveyard(item.controller)).first() {
        g.cast_during_resolution(card, None)?;
    }
    Ok(())
}

static FLASHBACK_INSTRUCTION: CardDef = CardDef {
    name: "Route Flashback Instruction Probe",
    cost: ManaCost::ZERO,
    spell: Some(SpellDef { targets: &[], effect: cast_own_graveyard_card }),
    ..PROBE_SPELL
};

static SPLIT_LEFT: CardDef = CardDef { name: "Route Split Left", ..PROBE_SPELL };
static SPLIT_RIGHT: CardDef = CardDef { name: "Route Split Right", cost: ManaCost { generic: 4, ..ManaCost::ZERO }, ..PROBE_SPELL };
static SPLIT: CardDef = CardDef { name: "Route Split Probe", oracle: true, split: Some((&SPLIT_LEFT, &SPLIT_RIGHT)), ..CardDef::BLANK };

fn any_top_spell(_: &dyn Query, _: &Asker, _: ObjectId) -> bool {
    true
}

static LIFE_TOP: CardDef = CardDef {
    name: "Route Library Life Source",
    types: TypeSet::of(&[CardType::Artifact]),
    play_rules: &[PlayRule::LibraryTop],
    ..CardDef::BLANK
};

static MANA_TOP: CardDef = CardDef {
    name: "Route Library Mana Source",
    types: TypeSet::of(&[CardType::Artifact]),
    play_rules: &[PlayRule::LibraryTopSpells(any_top_spell)],
    ..CardDef::BLANK
};

static PERMISSION_SOURCE: CardDef = CardDef { name: "Route Permission Source", types: TypeSet::of(&[CardType::Artifact]), ..CardDef::BLANK };

fn one_life(_: &dyn Query, _: &Asker, _: ObjectId, _: PlayerId) -> Option<Cost> {
    Some(Cost { life: 1, ..Cost::NONE })
}

fn nothing(_: &dyn Query, _: &Asker, _: ObjectId, _: PlayerId) -> Option<Cost> {
    Some(Cost::NONE)
}

static ONE_LIFE_WAY: GrantedWay = GrantedWay { text: "one life", cost: one_life, ..GrantedWay::CAST };
static FREE_FLASHBACK_WAY: GrantedWay = GrantedWay { text: "free flashback", cost: nothing, flashback: true, ..GrantedWay::CAST };

macro_rules! grantor {
    ($name:literal, $way:ident) => {
        CardDef { name: $name, types: TypeSet::of(&[CardType::Artifact]), granted_ways: &[$way], ..CardDef::BLANK }
    };
}

static GRANTOR_0: CardDef = grantor!("Route Grantor 0", ONE_LIFE_WAY);
static GRANTOR_1: CardDef = grantor!("Route Grantor 1", ONE_LIFE_WAY);
static GRANTOR_2: CardDef = grantor!("Route Grantor 2", ONE_LIFE_WAY);
static GRANTOR_3: CardDef = grantor!("Route Grantor 3", ONE_LIFE_WAY);
static GRANTOR_4: CardDef = grantor!("Route Grantor 4", ONE_LIFE_WAY);
static GRANTOR_5: CardDef = grantor!("Route Grantor 5", ONE_LIFE_WAY);
static GRANTOR_6: CardDef = grantor!("Route Grantor 6", ONE_LIFE_WAY);
static GRANTOR_7: CardDef = grantor!("Route Grantor 7", ONE_LIFE_WAY);
static GRANTOR_8: CardDef = grantor!("Route Grantor 8", FREE_FLASHBACK_WAY);

fn any_card(_: &dyn Query, _: &Asker, _: ObjectId, _: PlayerId) -> bool {
    true
}

fn spells_you_cast(_: &dyn Query, asker: &Asker, _: ObjectId, player: PlayerId) -> bool {
    player == asker.controller
}

static EXILE_PERMISSION: CardDef = CardDef {
    name: "Route Exile Permission",
    types: TypeSet::of(&[CardType::Artifact]),
    exile_casts: &[ExileCast { text: "Cast from exile", allows: any_card, additional: Cost::NONE, any_type: false }],
    ..CardDef::BLANK
};

static FREE_ALTERNATIVE: CardDef = CardDef {
    name: "Route Free Alternative",
    types: TypeSet::of(&[CardType::Artifact]),
    spell_alternatives: &[SpellAlternative {
        text: "Pay {0} rather than its mana cost",
        applies: spells_you_cast,
        cost: InsteadOfManaCost::Mana(ManaCost::ZERO),
        flash: false,
    }],
    ..CardDef::BLANK
};

static FILLER: CardDef = CardDef { name: "Route Filler", has_cost: false, ..CardDef::BLANK };

fn table() -> &'static CardTable {
    static TABLE: OnceLock<CardTable> = OnceLock::new();
    TABLE.get_or_init(|| {
        CardTable::new([
            &PROBE_SPELL,
            &WAY_SPELL,
            &INTRINSIC_GRAVEYARD_SPELL,
            &ADVENTURE_PROBE,
            &ADVENTURE_FACE,
            &RESOLUTION_PROBE,
            &FLASHBACK_SPELL,
            &FLASHBACK_INSTRUCTION,
            &SPLIT_LEFT,
            &SPLIT_RIGHT,
            &SPLIT,
            &LIFE_TOP,
            &MANA_TOP,
            &PERMISSION_SOURCE,
            &FILLER,
            &GRANTOR_0,
            &GRANTOR_1,
            &GRANTOR_2,
            &GRANTOR_3,
            &GRANTOR_4,
            &GRANTOR_5,
            &GRANTOR_6,
            &GRANTOR_7,
            &GRANTOR_8,
            &EXILE_PERMISSION,
            &FREE_ALTERNATIVE,
        ])
    })
}

/// A game in the first player's precombat main phase with priority, their first deck card where `to` says, `colorless`
/// mana in their pool, and the rest of both decks filler.
fn game_with(name: &str, to: fn(PlayerId) -> Zone, colorless: u8) -> (RulesGame, ObjectId) {
    let cards = table();
    let card = cards.find(name).unwrap();
    let filler = cards.find("Route Filler").unwrap();
    let mut game = RulesGame::new(cards, &Setup { decks: vec![vec![card, filler, filler, filler, filler, filler, filler], vec![filler; 7]], seed: 1, ..Setup::default() });
    let id = game.state.zone(Zone::hand(ME))[0];
    move_to(&mut game, id, to(ME));
    begin(&mut game, colorless);
    (game, id)
}

fn move_to(game: &mut RulesGame, id: ObjectId, zone: Zone) {
    let from = game.state.obj(id).zone;
    assert!(game.state.zone_remove(from, id));
    game.state.update_obj(id, |o| o.zone = zone);
    game.state.zone_push(zone, id);
}

fn begin(game: &mut RulesGame, colorless: u8) {
    game.state.update_turn(|t| {
        t.number = 1;
        t.active = ME;
        t.priority = ME;
        t.set_step(Step::PrecombatMain);
        t.began = true;
    });
    game.state.update_player(ME, |p| p.pool.0[Mana::C as usize] = colorless);
    game.state.frame_push(Frame::new(FrameKind::Priority));
}

fn put(game: &mut RulesGame, name: &str, player: PlayerId) -> ObjectId {
    let card = game.cards.find(name).unwrap();
    let id = game.state.push_obj(Obj::new(card, player, Zone::BATTLEFIELD));
    game.state.zone_push(Zone::BATTLEFIELD, id);
    id
}

fn permission(card: ObjectId, by: ObjectId, grant: PlayGrant) -> PlayPermission {
    PlayPermission { card, player: ME, grant, by: Grant { object: by, ability: AbilityRef::spell(CardId(0)) }, turn: 1 }
}

fn casts_of(game: &RulesGame, card: ObjectId) -> Vec<ActionOption> {
    match &game.pending().expect("a priority request").kind {
        RequestKind::Priority { options } => options.iter().filter(|o| o.kind == ActionKind::CastSpell && o.source == Some(card)).cloned().collect(),
        other => panic!("expected priority, got {other:?}"),
    }
}

fn by(option: &ActionOption) -> Option<ObjectId> {
    option.granted_by.map(|g| g.object)
}

fn take(game: &mut RulesGame, option: &ActionOption) {
    let request = game.pending().unwrap().clone();
    game.answer(request.id, request.player, Answer::Act(Action::of(option))).unwrap();
}

/// Plays on until the stack is empty again after a cast: passes, takes a printed way or another object's alternative cost
/// where a cost is to be chosen, and confirms a payment.
fn resolve(game: &mut RulesGame) {
    for _ in 0..24 {
        game.run();
        let cast = game.log.iter().any(|r| matches!(r.event, Event::SpellCast { player: ME, .. }));
        if game.state.stack.is_empty() && cast {
            return;
        }
        let request = game.pending().cloned().expect("asked before the cast resolves");
        let answer = match request.kind {
            RequestKind::Priority { .. } => Answer::Pass,
            RequestKind::ChooseModes { modes, eligible, .. } => {
                let picked = eligible.iter().copied().find(|&i| matches!(modes[i as usize], Label::Way(..) | Label::AlternativeCost(..)));
                Answer::Modes(vec![picked.or(eligible.first().copied()).expect("a mode")])
            }
            RequestKind::PayCosts { options, .. } => {
                let confirm = options
                    .iter()
                    .find(|o| o.kind == ActionKind::AssignMana)
                    .or_else(|| options.iter().find(|o| o.kind == ActionKind::ConfirmCasting))
                    .or_else(|| options.iter().find(|o| o.kind == ActionKind::CancelCasting))
                    .unwrap_or_else(|| panic!("unexpected payment options: {options:?}"));
                Answer::Act(Action::of(confirm))
            }
            other => panic!("unexpected request: {other:?}"),
        };
        game.answer(request.id, request.player, answer).unwrap();
    }
    panic!("the cast did not resolve");
}

fn in_zone(game: &RulesGame, zone: Zone, card: CardId) -> bool {
    game.state.zone(zone).iter().any(|&id| game.state.obj(id).card == card)
}

/// CR 608.2g, 702.34a: a card an effect has cast from its owner's graveyard as it resolves may be cast for its flashback
/// cost, and is exiled as it leaves the stack.
#[test]
fn a_cast_as_an_effect_resolves_may_use_the_cards_own_flashback_from_its_graveyard() {
    let cards = table();
    let (instruction, flashback, filler) = (cards.find("Route Flashback Instruction Probe").unwrap(), cards.find("Route Resolution Flashback Probe").unwrap(), cards.find("Route Filler").unwrap());
    let mut game = RulesGame::new(cards, &Setup { decks: vec![vec![instruction, flashback, filler, filler, filler, filler, filler], vec![filler; 7]], seed: 1, ..Setup::default() });
    let (instruction_id, flashback_id) = (game.state.zone(Zone::hand(ME))[0], game.state.zone(Zone::hand(ME))[1]);
    move_to(&mut game, flashback_id, Zone::graveyard(ME));
    begin(&mut game, 0);
    game.run();
    let option = casts_of(&game, instruction_id).into_iter().next().expect("the instruction is castable");
    take(&mut game, &option);
    resolve(&mut game);
    let cast = game.log.iter().find_map(|r| match r.event {
        Event::Cast(c) if c.from == ZoneKind::Graveyard => Some(c),
        _ => None,
    });
    assert_eq!(cast.expect("the graveyard card was cast").way, 1, "by its printed flashback");
    assert!(in_zone(&game, Zone::exile(ME), flashback) && !in_zone(&game, Zone::graveyard(ME), flashback));
}

/// CR 601.3, 118.9, 709.3a: a split card on top of the library is offered by each permission for each half - for life
/// equal to that half's mana value by one, for that half's mana cost by the other - and the half chosen is cast.
#[test]
fn each_permission_for_the_top_of_the_library_offers_each_split_half_at_its_own_cost() {
    let (mut game, top) = game_with("Route Split Probe", Zone::library, 4);
    let library = game.state.zone(Zone::library(ME)).to_vec();
    game.state.zone_remove(Zone::library(ME), top);
    game.state.zone_insert(Zone::library(ME), 0, top);
    assert_eq!(game.state.zone(Zone::library(ME)).len(), library.len());
    let life = put(&mut game, "Route Library Life Source", ME);
    let mana = put(&mut game, "Route Library Mana Source", ME);
    game.run();
    let options = casts_of(&game, top);
    let of = |source| options.iter().filter(|o| by(o) == Some(source)).collect::<Vec<_>>();
    let mut lives: Vec<u8> = of(life).iter().map(|o| cantrip_rules::life_for_cast(&game, game.cards, ME, o).expect("for life")).collect();
    lives.sort_unstable();
    assert_eq!(lives, [2, 4], "life equal to each half's own mana value");
    let by_mana = of(mana);
    assert_eq!(by_mana.len(), 2, "each half for mana");
    assert!(by_mana.iter().all(|o| cantrip_rules::life_for_cast(&game, game.cards, ME, o).is_none()));
    let four = (*by_mana.iter().find(|o| o.index == WAY_SPLIT_RIGHT).expect("the four-mana half")).clone();
    take(&mut game, &four);
    resolve(&mut game);
    assert!(game.log.iter().any(|r| matches!(r.event, Event::Cast(c) if c.from == ZoneKind::Library && c.value == 4 && c.granted_by.is_some_and(|g| g.object == mana))));
    assert_eq!(game.state.player(ME).life, 20);
}

/// CR 118.9a: a permission that names its own alternative cost (life for the mana value) opens no other; one that casts for
/// the costs the card has opens its alternative cost that names no zone.
#[test]
fn a_printed_alternative_cost_is_open_by_the_permission_that_names_none_only() {
    let (mut game, top) = game_with("Route Way Probe", Zone::library, 0);
    game.state.zone_remove(Zone::library(ME), top);
    game.state.zone_insert(Zone::library(ME), 0, top);
    let life = put(&mut game, "Route Library Life Source", ME);
    let mana = put(&mut game, "Route Library Mana Source", ME);
    game.run();
    let options = casts_of(&game, top);
    assert!(!options.iter().any(|o| by(o) == Some(life) && o.index == 1));
    assert!(options.iter().any(|o| by(o) == Some(mana) && o.index == 1));
    // The way naming the graveyard is no way from the library.
    assert!(!options.iter().any(|o| o.index == 2));
}

/// CR 601.2b, 601.3: two permissions for one card are two casts, whichever was given first, and the one chosen is the one
/// the cast is made by: for nothing, not for life, and with no flashback.
#[test]
fn two_play_permissions_are_two_casts_and_the_chosen_one_is_made() {
    for free_first in [true, false] {
        let (mut game, spell) = game_with("Route Probe", Zone::graveyard, 0);
        let (life_source, free_source) = (put(&mut game, "Route Permission Source", ME), put(&mut game, "Route Permission Source", ME));
        let life = permission(spell, life_source, PlayGrant { life_for_mana_value: true, exile_after: true, ..PlayGrant::CAST });
        let free = permission(spell, free_source, PlayGrant { instead: Some(ManaCost::ZERO), ..PlayGrant::CAST });
        for p in if free_first { [free, life] } else { [life, free] } {
            game.state.permission_push(p);
        }
        game.run();
        let options = casts_of(&game, spell);
        let free = options.iter().find(|o| by(o) == Some(free_source)).expect("the free permission's cast").clone();
        let life = options.iter().find(|o| by(o) == Some(life_source)).expect("the life permission's cast");
        assert_eq!(cantrip_rules::life_for_cast(&game, game.cards, ME, life), Some(2));
        assert_eq!(cantrip_rules::life_for_cast(&game, game.cards, ME, &free), None);
        take(&mut game, &free);
        resolve(&mut game);
        assert_eq!(game.state.player(ME).life, 20, "no life paid");
        assert!(in_zone(&game, Zone::graveyard(ME), table().find("Route Probe").unwrap()), "no flashback: back to the graveyard");
    }
}

/// CR 601.3: the card's own "you may cast this card from your graveyard", the effect of Yawgmoth's Will and the permission
/// a card exiled on an Adventure has each stay a cast of its own beside a permission that names a cost, and are made as
/// themselves: recorded so, and back to the graveyard rather than into the exile the other's flashback would send it to.
#[test]
fn a_rule_or_the_cards_own_permission_stays_beside_a_permission_that_names_a_cost() {
    let forced = PlayGrant { instead: Some(ManaCost::ZERO), exile_after: true, ..PlayGrant::CAST };
    for case in 0..3 {
        let (name, zone): (&str, fn(PlayerId) -> Zone) = match case {
            0 => ("Route Intrinsic Graveyard Probe", Zone::graveyard),
            1 => ("Route Probe", Zone::graveyard),
            _ => ("Route Adventure Probe", Zone::exile),
        };
        let (mut game, spell) = game_with(name, zone, 2);
        let source = put(&mut game, "Route Permission Source", ME);
        if case == 1 {
            game.state.effect_push(ContinuousEffect {
                source,
                controller: ME,
                timestamp: 1,
                affected: source,
                kind: EffectKind::PlayFromGraveyard,
                duration: Duration::Indefinite,
            });
        }
        if case == 2 {
            game.state.update_obj(spell, |o| o.on_adventure = Some(ME));
        }
        game.state.permission_push(permission(spell, source, forced));
        game.run();
        let options = casts_of(&game, spell);
        let forced_cast = options.iter().find(|o| o.index == 0 && game.state.permissions.iter().any(|p| Some(p.by) == o.granted_by));
        assert!(forced_cast.is_some(), "case {case}: the permission's own cast");
        let ordinary = options
            .iter()
            .find(|o| o.index == 0 && !game.state.permissions.iter().any(|p| Some(p.by) == o.granted_by))
            .unwrap_or_else(|| panic!("case {case}: the other permission's cast: {options:?}"))
            .clone();
        take(&mut game, &ordinary);
        resolve(&mut game);
        let card = game.state.obj(spell).card;
        assert!(in_zone(&game, Zone::graveyard(ME), card) && !in_zone(&game, Zone::exile(ME), card), "case {case}");
        assert!(game.log.iter().any(|r| matches!(r.event, Event::Cast(c) if c.granted_by == ordinary.granted_by && c.way == 0)), "case {case}");
    }
}

/// An action naming a permission the question did not offer it by is refused, and the question stays.
#[test]
fn a_cast_by_a_permission_not_offered_is_refused() {
    let (mut game, spell) = game_with("Route Probe", Zone::graveyard, 0);
    let source = put(&mut game, "Route Permission Source", ME);
    game.state.permission_push(permission(spell, source, PlayGrant { instead: Some(ManaCost::ZERO), ..PlayGrant::CAST }));
    game.run();
    let option = casts_of(&game, spell).into_iter().next().unwrap();
    let request = game.pending().unwrap().clone();
    let mut forged = Action::of(&option);
    forged.granted_by = Some(Grant { object: source, ability: AbilityRef::spell(CardId(1)) });
    assert!(game.answer(request.id, request.player, Answer::Act(forged.clone())).is_err());
    forged.granted_by = None;
    assert!(game.answer(request.id, request.player, Answer::Act(forged)).is_err());
    assert!(game.pending().is_some());
}

/// CR 118.9a: by a permission that names its own cost, the card's zone-free alternative cost is not offered; by one that
/// does not, it is; and the card's own way from its graveyard is a cast of its own either way.
#[test]
fn a_permission_naming_a_cost_borrows_no_alternative_cost() {
    let (mut game, spell) = game_with("Route Way Probe", Zone::graveyard, 0);
    let (imposed, ordinary) = (put(&mut game, "Route Permission Source", ME), put(&mut game, "Route Permission Source", ME));
    game.state.permission_push(permission(spell, imposed, PlayGrant { instead: Some(ManaCost::ZERO), ..PlayGrant::CAST }));
    game.state.permission_push(permission(spell, ordinary, PlayGrant::CAST));
    game.run();
    let options = casts_of(&game, spell);
    assert!(!options.iter().any(|o| by(o) == Some(imposed) && o.index == 1));
    assert!(options.iter().any(|o| by(o) == Some(ordinary) && o.index == 1));
    assert!(options.iter().any(|o| o.granted_by.is_none() && o.index == 2));
}

/// CR 608.2g, 601.3: an effect that has a card in another player's graveyard cast is the permission; a way a permanent
/// gives cards in its controller's own graveyard has nothing to do with it, so the card is cast for its mana cost or not
/// at all (here, with no mana, not).
#[test]
fn a_cast_as_an_effect_resolves_uses_no_way_a_permanent_gives() {
    let cards = table();
    let (resolver, spell, filler) = (cards.find("Route Resolution Probe").unwrap(), cards.find("Route Probe").unwrap(), cards.find("Route Filler").unwrap());
    let mut game = RulesGame::new(cards, &Setup { decks: vec![vec![resolver, filler, filler, filler, filler, filler, filler], vec![spell, filler, filler, filler, filler, filler, filler]], seed: 1, ..Setup::default() });
    let resolver_id = game.state.zone(Zone::hand(ME))[0];
    let target = game.state.zone(Zone::hand(PlayerId(1)))[0];
    move_to(&mut game, target, Zone::graveyard(PlayerId(1)));
    begin(&mut game, 0);
    put(&mut game, "Route Grantor 8", ME);
    game.run();
    let option = casts_of(&game, resolver_id).into_iter().next().unwrap();
    take(&mut game, &option);
    resolve(&mut game);
    assert_eq!(game.state.obj(target).zone, Zone::graveyard(PlayerId(1)));
}

/// CR 601.3, 118.9: a permanent's permission to cast from exile and another's alternative cost compose; a permission that
/// names its own cost composes with none (CR 118.9a). The cast is recorded as made by the permission.
#[test]
fn a_permission_and_another_objects_alternative_cost_compose() {
    let (mut game, spell) = game_with("Route Probe", Zone::exile, 0);
    let permitter = put(&mut game, "Route Exile Permission", ME);
    let payer = put(&mut game, "Route Free Alternative", ME);
    game.run();
    let options = casts_of(&game, spell);
    let combined = options.iter().find(|o| by(o) == Some(permitter) && o.paid_by.is_some_and(|g| g.object == payer)).expect("both").clone();
    assert!(!options.iter().any(|o| o.paid_by.is_some() && by(o) != Some(permitter)));
    take(&mut game, &combined);
    resolve(&mut game);
    assert!(game.log.iter().any(|r| matches!(r.event, Event::Cast(c) if c.granted_by.is_some_and(|g| g.object == permitter))));

    let (mut game, spell) = game_with("Route Probe", Zone::exile, 0);
    let source = put(&mut game, "Route Permission Source", ME);
    put(&mut game, "Route Free Alternative", ME);
    game.state.permission_push(permission(spell, source, PlayGrant { instead: Some(ManaCost { generic: 1, ..ManaCost::ZERO }), ..PlayGrant::CAST }));
    game.state.update_player(ME, |p| p.pool.0[Mana::C as usize] = 1);
    game.run();
    assert!(casts_of(&game, spell).iter().all(|o| o.paid_by.is_none()));
}

/// Nine permanents giving ways: each is a cast of its own (no slot limit), the last one's cast is for its own cost (nothing)
/// with its own flashback, and the history says which permanent it was made by.
#[test]
fn every_way_a_permanent_gives_is_a_cast_of_its_own_with_its_own_terms() {
    let (mut game, spell) = game_with("Route Probe", Zone::graveyard, 0);
    let grantors: Vec<ObjectId> = (0..9).map(|n| put(&mut game, &format!("Route Grantor {n}"), ME)).collect();
    game.run();
    let options = casts_of(&game, spell);
    for &g in &grantors {
        assert_eq!(options.iter().filter(|o| by(o) == Some(g)).count(), 1);
    }
    let first = options.iter().find(|o| by(o) == Some(grantors[0])).unwrap();
    assert_eq!(cantrip_rules::granted_cast_cost(&game, ME, first).map(|c| c.life), Some(1));
    let last = options.iter().find(|o| by(o) == Some(grantors[8])).unwrap().clone();
    take(&mut game, &last);
    resolve(&mut game);
    assert_eq!(game.state.player(ME).life, 20);
    assert!(in_zone(&game, Zone::exile(ME), table().find("Route Probe").unwrap()), "the way's flashback exiles it");
    assert!(game.log.iter().any(|r| matches!(r.event, Event::Cast(c) if c.granted_by.is_some_and(|g| g.object == grantors[8]))));
}

