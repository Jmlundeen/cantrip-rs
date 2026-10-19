//! A cast option carries the permission and cost route the player chose all the way through announcement.
//! The cards here are small probes for the general permission machinery, not card implementations.

use std::sync::OnceLock;

use cantrip_api::*;
use cantrip_rules::{Game as RulesGame, Setup, cast_plan_cost};
use cantrip_state::{CastCostChoice, CastPermission, Frame, FrameKind};

static PROBE_SPELL: CardDef = CardDef {
    name: "Cast Plan Probe",
    oracle: true,
    cost: ManaCost {
        generic: 2,
        ..ManaCost::ZERO
    },
    types: TypeSet::of(&[CardType::Instant]),
    spell: Some(SpellDef::MODAL),
    ..CardDef::BLANK
};

static HAND_COST_WAY: WayDef = WayDef {
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

static WAY_SPELL: CardDef = CardDef {
    name: "Cast Plan Way Probe",
    oracle: true,
    cost: ManaCost {
        generic: 2,
        ..ManaCost::ZERO
    },
    types: TypeSet::of(&[CardType::Instant]),
    spell: Some(SpellDef::MODAL),
    ways: &[HAND_COST_WAY, GRAVEYARD_WAY],
    ..CardDef::BLANK
};

static INTRINSIC_GRAVEYARD_SPELL: CardDef = CardDef {
    name: "Cast Plan Intrinsic Graveyard Probe",
    oracle: true,
    cost: ManaCost {
        generic: 2,
        ..ManaCost::ZERO
    },
    types: TypeSet::of(&[CardType::Instant]),
    keywords: Keywords::of(&[Keyword::CastFromYourGraveyard]),
    spell: Some(SpellDef::MODAL),
    ..CardDef::BLANK
};

static ADVENTURE_PROBE_FACE: CardDef = CardDef {
    name: "Cast Plan Adventure Probe Adventure",
    oracle: true,
    cost: ManaCost {
        generic: 1,
        ..ManaCost::ZERO
    },
    types: TypeSet::of(&[CardType::Sorcery]),
    spell: Some(SpellDef::MODAL),
    ..CardDef::BLANK
};

static ADVENTURE_PROBE: CardDef = CardDef {
    name: "Cast Plan Adventure Probe",
    oracle: true,
    cost: ManaCost {
        generic: 2,
        ..ManaCost::ZERO
    },
    types: TypeSet::of(&[CardType::Instant]),
    spell: Some(SpellDef::MODAL),
    adventure: Some(&ADVENTURE_PROBE_FACE),
    ..CardDef::BLANK
};

fn try_cast_opponent_graveyard_card(
    g: &mut dyn cantrip_api::Game,
    item: &StackItem,
) -> Result<(), Suspend> {
    let opponent = PlayerId(1 - item.controller.0);
    if let Some(&card) = g.state().zone(Zone::graveyard(opponent)).first() {
        let _ = g.cast_during_resolution(card, None)?;
    }
    Ok(())
}

static RESOLUTION_PROBE: CardDef = CardDef {
    name: "Cast Plan Resolution Probe",
    oracle: true,
    cost: ManaCost::ZERO,
    types: TypeSet::of(&[CardType::Instant]),
    spell: Some(SpellDef {
        targets: &[],
        effect: try_cast_opponent_graveyard_card,
    }),
    ..CardDef::BLANK
};

static RESOLUTION_FLASHBACK_WAY: WayDef = WayDef {
    text: "Flashback {0}",
    cost: Cost::mana(ManaCost::ZERO),
    from: ZoneKind::Graveyard,
    only_from: true,
    exile_after: true,
    condition: None,
    instant_speed: true,
    enters_attacking: false,
    kind: WayKind::Other,
};

static RESOLUTION_FLASHBACK_SPELL: CardDef = CardDef {
    name: "Cast Plan Resolution Flashback Probe",
    oracle: true,
    cost: ManaCost { generic: 2, ..ManaCost::ZERO },
    types: TypeSet::of(&[CardType::Instant]),
    spell: Some(SpellDef::MODAL),
    ways: &[RESOLUTION_FLASHBACK_WAY],
    ..CardDef::BLANK
};

fn cast_own_graveyard_card_during_resolution(
    g: &mut dyn cantrip_api::Game,
    item: &StackItem,
) -> Result<(), Suspend> {
    if let Some(&card) = g.state().zone(Zone::graveyard(item.controller)).first() {
        let _ = g.cast_during_resolution(card, None)?;
    }
    Ok(())
}

static RESOLUTION_FLASHBACK_INSTRUCTION: CardDef = CardDef {
    name: "Cast Plan Flashback Instruction Probe",
    oracle: true,
    cost: ManaCost::ZERO,
    types: TypeSet::of(&[CardType::Instant]),
    spell: Some(SpellDef {
        targets: &[],
        effect: cast_own_graveyard_card_during_resolution,
    }),
    ..CardDef::BLANK
};

static LIBRARY_SPLIT_LEFT: CardDef = CardDef {
    name: "Cast Plan Split Left",
    oracle: true,
    cost: ManaCost { generic: 2, ..ManaCost::ZERO },
    types: TypeSet::of(&[CardType::Instant]),
    spell: Some(SpellDef::MODAL),
    ..CardDef::BLANK
};

static LIBRARY_SPLIT_RIGHT: CardDef = CardDef {
    name: "Cast Plan Split Right",
    oracle: true,
    cost: ManaCost { generic: 4, ..ManaCost::ZERO },
    types: TypeSet::of(&[CardType::Instant]),
    spell: Some(SpellDef::MODAL),
    ..CardDef::BLANK
};

static LIBRARY_SPLIT: CardDef = CardDef {
    name: "Cast Plan Split Probe",
    oracle: true,
    split: Some((&LIBRARY_SPLIT_LEFT, &LIBRARY_SPLIT_RIGHT)),
    ..CardDef::BLANK
};

fn allows_any_top_spell(_: &dyn Query, _: &Asker, _: ObjectId) -> bool {
    true
}

static LIBRARY_LIFE_SOURCE: CardDef = CardDef {
    name: "Cast Plan Library Life Source",
    types: TypeSet::of(&[CardType::Artifact]),
    play_rules: &[PlayRule::LibraryTop],
    ..CardDef::BLANK
};

static LIBRARY_MANA_SOURCE: CardDef = CardDef {
    name: "Cast Plan Library Mana Source",
    types: TypeSet::of(&[CardType::Artifact]),
    play_rules: &[PlayRule::LibraryTopSpells(allows_any_top_spell)],
    ..CardDef::BLANK
};

static PROBE_SOURCE: CardDef = CardDef {
    name: "Permission Source Probe",
    types: TypeSet::of(&[CardType::Artifact]),
    ..CardDef::BLANK
};

fn one_life(_: &dyn Query, _: &Asker, _: ObjectId, _: PlayerId) -> Option<Cost> {
    Some(Cost {
        life: 1,
        ..Cost::mana(ManaCost::ZERO)
    })
}

fn no_cost(_: &dyn Query, _: &Asker, _: ObjectId, _: PlayerId) -> Option<Cost> {
    Some(Cost::mana(ManaCost::ZERO))
}

const fn granted_way(text: &'static str, cost: GrantedCost, flashback: bool) -> GrantedWay {
    GrantedWay {
        text,
        cost,
        permission_only: false,
        from: ZoneKind::Graveyard,
        any_zone: false,
        instant_speed: true,
        lands: false,
        gains: None,
        flashback,
    }
}

static ONE_LIFE_WAY: GrantedWay = granted_way("one life", one_life, false);
static FLASHBACK_WAY: GrantedWay = granted_way("free flashback", no_cost, true);

macro_rules! grantor {
    ($id:literal, $way:ident) => {
        CardDef {
            name: $id,
            types: TypeSet::of(&[CardType::Artifact]),
            granted_ways: &[$way],
            ..CardDef::BLANK
        }
    };
}

static GRANTOR_0: CardDef = grantor!("Cast Plan Grantor 0", ONE_LIFE_WAY);
static GRANTOR_1: CardDef = grantor!("Cast Plan Grantor 1", ONE_LIFE_WAY);
static GRANTOR_2: CardDef = grantor!("Cast Plan Grantor 2", ONE_LIFE_WAY);
static GRANTOR_3: CardDef = grantor!("Cast Plan Grantor 3", ONE_LIFE_WAY);
static GRANTOR_4: CardDef = grantor!("Cast Plan Grantor 4", ONE_LIFE_WAY);
static GRANTOR_5: CardDef = grantor!("Cast Plan Grantor 5", ONE_LIFE_WAY);
static GRANTOR_6: CardDef = grantor!("Cast Plan Grantor 6", ONE_LIFE_WAY);
static GRANTOR_7: CardDef = grantor!("Cast Plan Grantor 7", ONE_LIFE_WAY);
static GRANTOR_8: CardDef = grantor!("Cast Plan Grantor 8", FLASHBACK_WAY);

fn printed_cost(q: &dyn Query, _: &Asker, card: ObjectId, _: PlayerId) -> Option<Cost> {
    Some(Cost::mana(q.card(q.chars(card).card).cost))
}

static EXILE_PERMISSION: GrantedWay = GrantedWay {
    text: "exile permission",
    cost: printed_cost,
    from: ZoneKind::Exile,
    permission_only: true,
    ..GrantedWay::CAST
};
static EXILE_REQUIRED_COST: GrantedWay = GrantedWay {
    permission_only: false,
    ..EXILE_PERMISSION
};
static LIFE_ALTERNATIVE: GrantedWay = GrantedWay {
    text: "pay one life",
    cost: one_life,
    any_zone: true,
    ..GrantedWay::CAST
};
static EXILE_PERMISSION_SOURCE: CardDef = grantor!("Exile Permission Probe", EXILE_PERMISSION);
static EXILE_REQUIRED_COST_SOURCE: CardDef = grantor!("Exile Required Cost Probe", EXILE_REQUIRED_COST);
static LIFE_ALTERNATIVE_SOURCE: CardDef = grantor!("Life Alternative Probe", LIFE_ALTERNATIVE);

static FILLER: CardDef = CardDef {
    name: "Cast Plan Filler",
    has_cost: false,
    ..CardDef::BLANK
};

fn table() -> &'static CardTable {
    static TABLE: OnceLock<CardTable> = OnceLock::new();
    TABLE.get_or_init(|| {
        CardTable::new([
            &PROBE_SPELL,
            &WAY_SPELL,
            &INTRINSIC_GRAVEYARD_SPELL,
            &ADVENTURE_PROBE,
            &ADVENTURE_PROBE_FACE,
            &RESOLUTION_PROBE,
            &RESOLUTION_FLASHBACK_SPELL,
            &RESOLUTION_FLASHBACK_INSTRUCTION,
            &LIBRARY_SPLIT_LEFT,
            &LIBRARY_SPLIT_RIGHT,
            &LIBRARY_SPLIT,
            &LIBRARY_LIFE_SOURCE,
            &LIBRARY_MANA_SOURCE,
            &PROBE_SOURCE,
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
            &EXILE_PERMISSION_SOURCE,
            &EXILE_REQUIRED_COST_SOURCE,
            &LIFE_ALTERNATIVE_SOURCE,
        ])
    })
}

fn game_with_spell_in_graveyard() -> (RulesGame, ObjectId) {
    game_with_card_in_graveyard("Cast Plan Probe")
}

fn game_with_card_in_graveyard(name: &str) -> (RulesGame, ObjectId) {
    game_with_card_in_graveyard_with_colorless(name, 0)
}

fn game_with_card_in_graveyard_with_colorless(name: &str, mana: u8) -> (RulesGame, ObjectId) {
    let cards = table();
    let spell_card = cards.find(name).unwrap();
    let filler = cards.find("Cast Plan Filler").unwrap();
    let mut game = RulesGame::new(
        cards,
        &Setup {
            decks: vec![
                vec![spell_card, filler, filler, filler, filler, filler, filler],
                vec![filler; 7],
            ],
            seed: 1,
            ..Setup::default()
        },
    );
    let player = PlayerId(0);
    let spell = game.state.zone(Zone::hand(player))[0];
    assert!(game.state.zone_remove(Zone::hand(player), spell));
    game.state
        .update_obj(spell, |object| object.zone = Zone::graveyard(player));
    game.state.zone_push(Zone::graveyard(player), spell);
    game.state.update_turn(|turn| {
        turn.number = 1;
        turn.active = player;
        turn.priority = player;
        turn.set_step(Step::PrecombatMain);
        turn.began = true;
    });
    game.state.update_player(player, |state| {
        state.pool.0[Mana::C as usize] = mana;
    });
    game.state.frame_push(Frame::new(FrameKind::Priority));
    (game, spell)
}

fn source(game: &mut RulesGame) -> ObjectId {
    let card = game.cards.find("Permission Source Probe").unwrap();
    let id = game
        .state
        .push_obj(Obj::new(card, PlayerId(0), Zone::BATTLEFIELD));
    game.state.zone_push(Zone::BATTLEFIELD, id);
    id
}

fn permission(card: ObjectId, source: ObjectId, grant: PlayGrant) -> PlayPermission {
    PlayPermission {
        card,
        player: PlayerId(0),
        grant,
        source: Some(source),
        turn: 1,
    }
}

fn permission_options(game: &RulesGame, spell: ObjectId) -> Vec<ActionOption> {
    match &game.pending().expect("priority request").kind {
        RequestKind::Priority { options } => options
            .iter()
            .filter(|option| option.kind == ActionKind::CastSpell && option.source == Some(spell))
            .cloned()
            .collect(),
        other => panic!("expected priority, got {other:?}"),
    }
}

fn resolve_free_cast(game: &mut RulesGame) {
    for _ in 0..24 {
        game.run();
        if game.state.stack.is_empty()
            && game.log.iter().any(|record| {
                matches!(
                    record.event,
                    Event::SpellCast {
                        player: PlayerId(0),
                        ..
                    }
                )
            })
        {
            return;
        }
        let request = game
            .pending()
            .cloned()
            .expect("game asks before the cast resolves");
        let answer = match request.kind {
            RequestKind::Priority { .. } => Answer::Pass,
            RequestKind::ChooseModes {
                modes, eligible, ..
            } => {
                let selected = eligible
                    .iter()
                    .copied()
                    .find(|&index| {
                        matches!(
                            modes[index as usize],
                            Label::GrantedWay(_, _) | Label::PrintedWay(_, _)
                        )
                    })
                    .or_else(|| eligible.first().copied())
                    .expect("at least one offered mode");
                Answer::Modes(vec![selected])
            }
            RequestKind::PayCosts { options, .. } => {
                let confirm = options
                    .iter()
                    .find(|option| option.kind == ActionKind::ConfirmCasting)
                    .or_else(|| {
                        options
                            .iter()
                            .find(|option| option.kind == ActionKind::CancelCasting)
                    })
                    .unwrap_or_else(|| panic!("unexpected payment options: {options:?}"));
                Answer::Act(Action::of(confirm))
            }
            other => panic!("unexpected cast request: {other:?}"),
        };
        game.answer(request.id, request.player, answer).unwrap();
    }
    panic!("the selected free cast did not resolve");
}

/// A cast instructed during resolution can choose its own flashback route; flashback's exile consequence then applies.
#[test]
fn resolution_cast_can_choose_its_printed_flashback_and_exiles_afterward() {
    let cards = table();
    let instruction = cards.find("Cast Plan Flashback Instruction Probe").unwrap();
    let flashback = cards.find("Cast Plan Resolution Flashback Probe").unwrap();
    let filler = cards.find("Cast Plan Filler").unwrap();
    let mut game = RulesGame::new(
        cards,
        &Setup {
            decks: vec![
                vec![instruction, flashback, filler, filler, filler, filler, filler],
                vec![filler; 7],
            ],
            seed: 1,
            ..Setup::default()
        },
    );
    let player = PlayerId(0);
    let instruction_id = game.state.zone(Zone::hand(player))[0];
    let flashback_id = game.state.zone(Zone::hand(player))[1];
    assert!(game.state.zone_remove(Zone::hand(player), flashback_id));
    game.state
        .update_obj(flashback_id, |object| object.zone = Zone::graveyard(player));
    game.state.zone_push(Zone::graveyard(player), flashback_id);
    game.state.update_turn(|turn| {
        turn.number = 1;
        turn.active = player;
        turn.priority = player;
        turn.set_step(Step::PrecombatMain);
        turn.began = true;
    });
    game.state.frame_push(Frame::new(FrameKind::Priority));
    game.run();

    let request = game.pending().expect("priority request").clone();
    let option = match &request.kind {
        RequestKind::Priority { options } => options
            .iter()
            .find(|option| {
                option.kind == ActionKind::CastSpell
                    && option.source == Some(instruction_id)
            })
            .expect("the resolution instruction is castable")
            .clone(),
        other => panic!("expected priority, got {other:?}"),
    };
    game.answer(request.id, request.player, Answer::Act(Action::of(&option)))
        .unwrap();
    resolve_free_cast(&mut game);

    let cast = game.log.iter().find_map(|record| match record.event {
        Event::Cast(cast) if cast.spell != instruction_id && cast.from == ZoneKind::Graveyard => {
            Some(cast)
        }
        _ => None,
    });
    assert!(cast.is_some(), "the resolving effect must cast the graveyard card");
    assert_eq!(cast.unwrap().way, 1, "the cast used the printed flashback way");
    assert!(game.state.zone(Zone::exile(player)).iter().any(|&id| {
        game.state.obj(id).card == flashback
    }), "a spell cast with flashback is exiled when it leaves the stack");
    assert!(!game.state.zone(Zone::graveyard(player)).iter().any(|&id| {
        game.state.obj(id).card == flashback
    }));
}

/// Split-face library routes keep each permission's cost attached to each face's own mana value.
#[test]
fn library_split_faces_keep_their_cost_for_each_permission_route() {
    let cards = table();
    let split_card = cards.find("Cast Plan Split Probe").unwrap();
    let filler = cards.find("Cast Plan Filler").unwrap();
    let mut game = RulesGame::new(
        cards,
        &Setup {
            decks: vec![
                vec![filler, filler, filler, filler, filler, filler, filler, split_card],
                vec![filler; 7],
            ],
            seed: 1,
            ..Setup::default()
        },
    );
    let player = PlayerId(0);
    let top = *game.state.zone(Zone::library(player)).first().unwrap();
    let life_source = game
        .state
        .push_obj(Obj::new(cards.find("Cast Plan Library Life Source").unwrap(), player, Zone::BATTLEFIELD));
    game.state.zone_push(Zone::BATTLEFIELD, life_source);
    let mana_source = game
        .state
        .push_obj(Obj::new(cards.find("Cast Plan Library Mana Source").unwrap(), player, Zone::BATTLEFIELD));
    game.state.zone_push(Zone::BATTLEFIELD, mana_source);
    game.state.update_turn(|turn| {
        turn.number = 1;
        turn.active = player;
        turn.priority = player;
        turn.set_step(Step::PrecombatMain);
        turn.began = true;
    });
    game.state.update_player(player, |state| {
        state.pool.0[Mana::C as usize] = 4;
    });
    game.state.frame_push(Frame::new(FrameKind::Priority));
    game.run();

    let options = permission_options(&game, top);
    let life_routes: Vec<_> = options
        .iter()
        .filter_map(|option| {
            let plan = option.cast_plan?;
            (plan.permission == (CastPermission::LibraryTop { source: life_source, rule: 0 }))
                .then_some((option, plan))
        })
        .collect();
    let mana_routes: Vec<_> = options
        .iter()
        .filter_map(|option| {
            let plan = option.cast_plan?;
            (plan.permission == (CastPermission::LibraryTop { source: mana_source, rule: 0 }))
                .then_some((option, plan))
        })
        .collect();
    assert_eq!(life_routes.len(), 2, "Citadel-style permission offers each split face");
    assert_eq!(mana_routes.len(), 2, "Forge-style permission independently offers each split face");

    let mut seen_life_values = Vec::new();
    let mut seen_mana_costs = Vec::new();
    for (_, plan) in life_routes {
        assert_eq!(plan.cost, CastCostChoice::LifeForManaValue);
        let cost = cast_plan_cost(&game, player, top, plan).unwrap();
        seen_life_values.push(cost.life);
    }
    let mut four_mana_route = None;
    for (option, plan) in mana_routes {
        assert_eq!(plan.cost, CastCostChoice::Printed);
        let cost = cast_plan_cost(&game, player, top, plan).unwrap();
        seen_mana_costs.push(cost.mana.generic);
        assert_eq!(cost.life, 0);
        if cost.mana.generic == 4 {
            four_mana_route = Some(option.clone());
        }
    }
    seen_life_values.sort_unstable();
    seen_mana_costs.sort_unstable();
    assert_eq!(seen_life_values, [2, 4]);
    assert_eq!(seen_mana_costs, [2, 4]);

    let request = game.pending().unwrap().clone();
    let route = four_mana_route.expect("the more expensive face is selectable for mana");
    game.answer(request.id, request.player, Answer::Act(Action::of(&route)))
        .unwrap();
    resolve_free_cast(&mut game);
    assert!(game.log.iter().any(|record| matches!(record.event,
        Event::Cast(cast) if cast.from == ZoneKind::Library && cast.value == 4
    )), "executing the selected route casts the four-mana split face from the library");
}

/// A cost-taking LibraryTopSpells grant may use a printed alternative; life-for-mana-value must not borrow it.
#[test]
fn library_printed_alternative_is_bound_to_the_non_life_permission() {
    let cards = table();
    let way_card = cards.find("Cast Plan Way Probe").unwrap();
    let filler = cards.find("Cast Plan Filler").unwrap();
    let mut game = RulesGame::new(
        cards,
        &Setup {
            decks: vec![
                vec![filler, filler, filler, filler, filler, filler, filler, way_card],
                vec![filler; 7],
            ],
            seed: 1,
            ..Setup::default()
        },
    );
    let player = PlayerId(0);
    let top = *game.state.zone(Zone::library(player)).first().unwrap();
    let life_source = game
        .state
        .push_obj(Obj::new(cards.find("Cast Plan Library Life Source").unwrap(), player, Zone::BATTLEFIELD));
    game.state.zone_push(Zone::BATTLEFIELD, life_source);
    let mana_source = game
        .state
        .push_obj(Obj::new(cards.find("Cast Plan Library Mana Source").unwrap(), player, Zone::BATTLEFIELD));
    game.state.zone_push(Zone::BATTLEFIELD, mana_source);
    game.state.update_turn(|turn| {
        turn.number = 1;
        turn.active = player;
        turn.priority = player;
        turn.set_step(Step::PrecombatMain);
        turn.began = true;
    });
    game.state.frame_push(Frame::new(FrameKind::Priority));
    game.run();

    let options = permission_options(&game, top);
    let under_life = options.iter().any(|option| option.cast_plan.is_some_and(|plan| {
        plan.permission == (CastPermission::LibraryTop { source: life_source, rule: 0 })
            && plan.cost == (CastCostChoice::PrintedWay { way: 0 })
    }));
    let under_mana = options.iter().any(|option| option.cast_plan.is_some_and(|plan| {
        plan.permission == (CastPermission::LibraryTop { source: mana_source, rule: 0 })
            && plan.cost == (CastCostChoice::PrintedWay { way: 0 })
    }));
    assert!(!under_life, "life-for-mana-value cannot be mixed with a printed alternative cost");
    assert!(under_mana, "an independent cost-taking permission retains the printed alternative route");
}

fn overlapping_permissions(free_first: bool) -> (RulesGame, ObjectId, u32) {
    let (mut game, spell) = game_with_spell_in_graveyard();
    let life_source = source(&mut game);
    let free_source = source(&mut game);
    let life = PlayGrant {
        life_for_mana_value: true,
        exile_after: true,
        ..PlayGrant::CAST
    };
    let free = PlayGrant {
        instead: Some(ManaCost::ZERO),
        ..PlayGrant::CAST
    };
    let permissions = if free_first {
        [
            permission(spell, free_source, free),
            permission(spell, life_source, life),
        ]
    } else {
        [
            permission(spell, life_source, life),
            permission(spell, free_source, free),
        ]
    };
    game.state.permission_push(permissions[0]);
    game.state.permission_push(permissions[1]);
    game.run();
    let free_index = if free_first { 0 } else { 1 };
    (game, spell, free_index)
}

/// CR 601.2b and 118.9: choose among permissions for the same card, and carry that exact cost and consequence through resolution.
#[test]
fn overlapping_play_permissions_execute_the_selected_route_in_either_order() {
    for free_first in [true, false] {
        let (mut game, spell, free_index) = overlapping_permissions(free_first);
        let player = PlayerId(0);
        let options = permission_options(&game, spell);
        let free = options.iter().find(|option| {
            option.cast_plan.is_some_and(|plan| matches!(plan.permission, CastPermission::PlayPermission { index } if index == free_index))
        }).expect("the cost-for-zero permission is offered");
        let life_index = if free_first { 1 } else { 0 };
        let life = options.iter().find(|option| {
            option.cast_plan.is_some_and(|plan| matches!(plan.permission, CastPermission::PlayPermission { index } if index == life_index))
        }).expect("the life-for-mana-value permission is also offered");

        let free_cost = cast_plan_cost(&game, player, spell, free.cast_plan.unwrap()).unwrap();
        assert_eq!(free_cost.mana, ManaCost::ZERO);
        let life_cost = cast_plan_cost(&game, player, spell, life.cast_plan.unwrap()).unwrap();
        assert_eq!(life_cost.life, 2);

        let request = game.pending().unwrap().clone();
        game.answer(request.id, request.player, Answer::Act(Action::of(free)))
            .unwrap();
        resolve_free_cast(&mut game);

        assert_eq!(
            game.state.player(player).life,
            20,
            "the selected route did not pay life"
        );
        assert!(
            game.state
                .zone(Zone::graveyard(player))
                .iter()
                .any(|&id| game.state.obj(id).card == table().find("Cast Plan Probe").unwrap()),
            "the selected non-flashback route returns the resolved spell to its graveyard"
        );
    }
}

/// An independent intrinsic graveyard permission survives a simultaneous forced-cost grant and executes as the printed cost.
#[test]
fn intrinsic_graveyard_cast_remains_selectable_beside_a_forced_cost_permission() {
    let (mut game, spell) = game_with_card_in_graveyard_with_colorless(
        "Cast Plan Intrinsic Graveyard Probe",
        2,
    );
    let player = PlayerId(0);
    let grant_source = source(&mut game);
    let forced = PlayGrant {
        instead: Some(ManaCost::ZERO),
        exile_after: true,
        ..PlayGrant::CAST
    };
    game.state
        .permission_push(permission(spell, grant_source, forced));
    game.run();

    let options = permission_options(&game, spell);
    let intrinsic = options
        .iter()
        .find(|option| {
            option.cast_plan.is_some_and(|plan| {
                plan.permission == CastPermission::PrintedGraveyard
                    && plan.cost == CastCostChoice::Printed
            })
        })
        .unwrap_or_else(|| panic!("the card's intrinsic graveyard permission remains a distinct route: {:?}", options.iter().filter_map(|o| o.cast_plan).collect::<Vec<_>>()));
    assert!(options.iter().any(|option| {
        option.cast_plan.is_some_and(|plan| {
            plan.permission == (CastPermission::PlayPermission { index: 0 })
        })
    }), "the forced-cost permission is independently offered");
    let cost = cast_plan_cost(&game, player, spell, intrinsic.cast_plan.unwrap()).unwrap();
    assert_eq!(cost.mana.generic, 2);

    let request = game.pending().expect("priority request").clone();
    game.answer(request.id, request.player, Answer::Act(Action::of(intrinsic)))
        .unwrap();
    resolve_free_cast(&mut game);

    assert_eq!(game.state.player(player).life, 20);
    assert!(game.state.zone(Zone::graveyard(player)).iter().any(|&id| {
        game.state.obj(id).card == game.state.obj(spell).card
    }));
    assert!(!game.state.zone(Zone::exile(player)).iter().any(|&id| {
        game.state.obj(id).card == game.state.obj(spell).card
    }));
    assert!(game.log.iter().any(|record| matches!(record.event,
        Event::Cast(history) if history.spell != spell && history.from == ZoneKind::Graveyard)));
}

/// The global PlayFromGraveyard rule remains an independent route beside a forced cost grant.
#[test]
fn global_graveyard_play_remains_selectable_beside_a_forced_cost_permission() {
    let (mut game, spell) = game_with_card_in_graveyard_with_colorless("Cast Plan Probe", 2);
    let player = PlayerId(0);
    let grant_source = source(&mut game);
    game.state.effect_push(ContinuousEffect {
        source: grant_source,
        controller: player,
        timestamp: 1,
        affected: grant_source,
        kind: EffectKind::PlayFromGraveyard,
        duration: Duration::Indefinite,
    });
    game.state.permission_push(permission(
        spell,
        grant_source,
        PlayGrant {
            instead: Some(ManaCost::ZERO),
            exile_after: true,
            ..PlayGrant::CAST
        },
    ));
    game.run();

    let options = permission_options(&game, spell);
    let ordinary = options
        .iter()
        .find(|option| option.cast_plan.is_some_and(|plan| {
            plan.permission == CastPermission::GraveyardPlayRule
                && plan.cost == CastCostChoice::Printed
        }))
        .unwrap_or_else(|| panic!("global graveyard permission was lost: {:?}", options.iter().filter_map(|o| o.cast_plan).collect::<Vec<_>>()));
    assert!(options.iter().any(|option| option.cast_plan.is_some_and(|plan| {
        plan.permission == (CastPermission::PlayPermission { index: 0 })
    })));
    assert_eq!(cast_plan_cost(&game, player, spell, ordinary.cast_plan.unwrap()).unwrap().mana.generic, 2);
    let request = game.pending().unwrap().clone();
    game.answer(request.id, request.player, Answer::Act(Action::of(ordinary))).unwrap();
    resolve_free_cast(&mut game);

    let card = game.state.obj(spell).card;
    assert!(game.state.zone(Zone::graveyard(player)).iter().any(|&id| game.state.obj(id).card == card));
    assert!(!game.state.zone(Zone::exile(player)).iter().any(|&id| game.state.obj(id).card == card));
}

/// The Adventure return permission in exile remains selectable beside an unrelated forced cost grant.
#[test]
fn adventure_exile_cast_remains_selectable_beside_a_forced_cost_permission() {
    let (mut game, spell) = game_with_card_in_graveyard_with_colorless("Cast Plan Adventure Probe", 2);
    let player = PlayerId(0);
    assert!(game.state.zone_remove(Zone::graveyard(player), spell));
    game.state.update_obj(spell, |object| {
        object.zone = Zone::exile(player);
        object.on_adventure = Some(PlayerId(0));
    });
    game.state.zone_push(Zone::exile(player), spell);
    let grant_source = source(&mut game);
    game.state.permission_push(permission(
        spell,
        grant_source,
        PlayGrant {
            instead: Some(ManaCost::ZERO),
            exile_after: true,
            ..PlayGrant::CAST
        },
    ));
    game.run();

    let options = permission_options(&game, spell);
    let ordinary = options
        .iter()
        .find(|option| option.cast_plan.is_some_and(|plan| {
            plan.permission == CastPermission::AdventureExile
                && plan.cost == CastCostChoice::Printed
        }))
        .unwrap_or_else(|| panic!("Adventure exile permission was lost: {:?}", options.iter().filter_map(|o| o.cast_plan).collect::<Vec<_>>()));
    assert!(options.iter().any(|option| option.cast_plan.is_some_and(|plan| {
        plan.permission == (CastPermission::PlayPermission { index: 0 })
    })));
    assert_eq!(cast_plan_cost(&game, player, spell, ordinary.cast_plan.unwrap()).unwrap().mana.generic, 2);
    let request = game.pending().unwrap().clone();
    game.answer(request.id, request.player, Answer::Act(Action::of(ordinary))).unwrap();
    resolve_free_cast(&mut game);

    let card = game.state.obj(spell).card;
    assert!(game.state.zone(Zone::graveyard(player)).iter().any(|&id| game.state.obj(id).card == card));
    assert!(!game.state.zone(Zone::exile(player)).iter().any(|&id| game.state.obj(id).card == card));
}

/// A forged selector must not turn an offered cast into a different permission or cost route.
#[test]
fn a_forged_cast_plan_is_rejected_by_the_outstanding_priority_request() {
    let (mut game, spell, free_index) = overlapping_permissions(false);
    let request = game.pending().unwrap().clone();
    let option = permission_options(&game, spell).into_iter().find(|option| {
        option.cast_plan.is_some_and(|plan| matches!(plan.permission, CastPermission::PlayPermission { index } if index == free_index))
    }).unwrap();
    let mut forged = Action::of(&option);
    forged.cast_plan.as_mut().unwrap().permission = CastPermission::PlayPermission { index: 99 };
    let rejected = game.answer(request.id, request.player, Answer::Act(forged));
    assert!(
        rejected.is_err(),
        "the engine accepted a permission selector absent from the offered options"
    );
    assert!(
        game.pending().is_some(),
        "rejecting the forged plan must leave the original request outstanding"
    );
}

/// A permission imposing a cost cannot borrow a printed alternative cost; an independent permission can.
#[test]
fn an_imposed_cost_route_cannot_borrow_a_printed_alternative_from_another_route() {
    let (mut game, spell) = game_with_card_in_graveyard("Cast Plan Way Probe");
    let imposed_source = source(&mut game);
    let ordinary_source = source(&mut game);
    let imposed = PlayGrant {
        instead: Some(ManaCost::ZERO),
        ..PlayGrant::CAST
    };
    let ordinary = PlayGrant::CAST;
    game.state
        .permission_push(permission(spell, imposed_source, imposed));
    game.state
        .permission_push(permission(spell, ordinary_source, ordinary));
    game.run();

    let options = permission_options(&game, spell);
    let imposed_has_hand_way = options.iter().any(|option| {
        option.cast_plan.is_some_and(|plan| {
            matches!(plan.permission, CastPermission::PlayPermission { index: 0 })
                && matches!(plan.cost, CastCostChoice::PrintedWay { way: 0 })
        })
    });
    let ordinary_has_hand_way = options.iter().any(|option| {
        option.cast_plan.is_some_and(|plan| {
            matches!(plan.permission, CastPermission::PlayPermission { index: 1 })
                && matches!(plan.cost, CastCostChoice::PrintedWay { way: 0 })
        })
    });
    let own_zone_way = options.iter().any(|option| {
        option.cast_plan.is_some_and(|plan| {
            matches!(plan.permission, CastPermission::PrintedWay { way: 1 })
                && matches!(plan.cost, CastCostChoice::PrintedWay { way: 1 })
        })
    });
    assert!(
        !imposed_has_hand_way,
        "the forced cost route must not borrow the hand-named printed cost"
    );
    assert!(
        ordinary_has_hand_way,
        "the independent printed-zone permission can offer a zone-neutral hand-named cost"
    );
    assert!(
        own_zone_way,
        "the spell's own graveyard way remains independently available"
    );
}

/// A fixed-zone GrantedWay is tied to the caster's own graveyard, even during a resolving instruction.
#[test]
fn resolution_cast_cannot_use_a_fixed_zone_grant_on_an_opponents_graveyard_card() {
    let cards = table();
    let resolver = cards.find("Cast Plan Resolution Probe").unwrap();
    let spell_card = cards.find("Cast Plan Probe").unwrap();
    let filler = cards.find("Cast Plan Filler").unwrap();
    let mut game = RulesGame::new(
        cards,
        &Setup {
            decks: vec![
                vec![resolver, filler, filler, filler, filler, filler, filler],
                vec![spell_card, filler, filler, filler, filler, filler, filler],
            ],
            seed: 1,
            ..Setup::default()
        },
    );
    let caster = PlayerId(0);
    let opponent = PlayerId(1);
    let resolver_id = game.state.zone(Zone::hand(caster))[0];
    let target = game.state.zone(Zone::hand(opponent))[0];
    assert!(game.state.zone_remove(Zone::hand(opponent), target));
    game.state
        .update_obj(target, |object| object.zone = Zone::graveyard(opponent));
    game.state.zone_push(Zone::graveyard(opponent), target);
    game.state.update_turn(|turn| {
        turn.number = 1;
        turn.active = caster;
        turn.priority = caster;
        turn.set_step(Step::PrecombatMain);
        turn.began = true;
    });
    game.state.frame_push(Frame::new(FrameKind::Priority));
    let grantor = source(&mut game);
    game.run();

    let request = game.pending().expect("priority request").clone();
    let option = match &request.kind {
        RequestKind::Priority { options } => options
            .iter()
            .find(|option| {
                option.kind == ActionKind::CastSpell && option.source == Some(resolver_id)
            })
            .expect("the resolving probe can be cast")
            .clone(),
        other => panic!("expected priority, got {other:?}"),
    };
    game.answer(request.id, request.player, Answer::Act(Action::of(&option)))
        .unwrap();
    resolve_free_cast(&mut game);

    assert_eq!(game.state.obj(target).zone, Zone::graveyard(opponent));
    assert!(
        !game.log.iter().any(|record| matches!(record.event,
        Event::SpellCast { player: PlayerId(0), spell, .. }
            if game.state.obj(spell).card == spell_card)),
        "opponent graveyard card was cast: {:?}",
        game.log
            .iter()
            .map(|record| &record.event)
            .collect::<Vec<_>>()
    );
    assert!(game.state.obj(grantor).zone.is(ZoneKind::Battlefield));
}

/// Independent permission and payment grants compose for any spell type, zone and cost resource (CR 118.9a).
#[test]
fn independent_grants_cast_a_noncreature_from_exile_for_life() {
    for (permission_name, allowed) in [("Exile Permission Probe", true), ("Exile Required Cost Probe", false)] {
        let (mut game, spell) = game_with_spell_in_graveyard();
        let player = PlayerId(0);
        assert!(game.state.zone_remove(Zone::graveyard(player), spell));
        game.state.update_obj(spell, |object| object.zone = Zone::exile(player));
        game.state.zone_push(Zone::exile(player), spell);
        let mut add_source = |name: &str| {
            let card = game.cards.find(name).unwrap();
            let id = game.state.push_obj(Obj::new(card, player, Zone::BATTLEFIELD));
            game.state.zone_push(Zone::BATTLEFIELD, id);
            id
        };
        let permission = add_source(permission_name);
        let payment = add_source("Life Alternative Probe");
        game.run();
        let options = permission_options(&game, spell);
        let combined = options.iter().find(|option| option.cast_plan.is_some_and(|plan|
            plan.permission == CastPermission::GrantedWay { source: permission, rule: 0 }
                && plan.cost == CastCostChoice::GrantedWay { source: payment, rule: 0 }));
        assert_eq!(combined.is_some(), allowed, "{permission_name}");
        if let Some(option) = combined {
            let cost = cast_plan_cost(&game, player, spell, option.cast_plan.unwrap()).unwrap();
            assert_eq!(cost.life, 1);
            assert_eq!(cost.mana, ManaCost::ZERO);
            assert_eq!(cost.energy, 0);
            let request = game.pending().unwrap().clone();
            game.answer(request.id, player, Answer::Act(Action::of(option))).unwrap();
            resolve_free_cast(&mut game);
            assert_eq!(game.state.player(player).life, 19);
            for source in [permission, payment] {
                assert_eq!(game.log.iter().filter(|record| matches!(record.event,
                    Event::AbilityActivated { source: used, ability, .. }
                        if used == source && ability.index == GRANTED_WAY_USED)).count(), 1);
            }
        }
    }
}

/// A ninth granted route keeps its own fixed cost and flashback consequence, and is executed as that route.
#[test]
fn the_ninth_granted_way_source_keeps_its_cost_and_flashback_consequence() {
    let (mut game, spell) = game_with_spell_in_graveyard();
    let names = [
        "Cast Plan Grantor 0",
        "Cast Plan Grantor 1",
        "Cast Plan Grantor 2",
        "Cast Plan Grantor 3",
        "Cast Plan Grantor 4",
        "Cast Plan Grantor 5",
        "Cast Plan Grantor 6",
        "Cast Plan Grantor 7",
        "Cast Plan Grantor 8",
    ];
    let sources: Vec<_> = names
        .iter()
        .map(|name| {
            let card = game.cards.find(name).unwrap();
            let id = game
                .state
                .push_obj(Obj::new(card, PlayerId(0), Zone::BATTLEFIELD));
            game.state.zone_push(Zone::BATTLEFIELD, id);
            id
        })
        .collect();
    game.run();

    let options = permission_options(&game, spell);
    let routes: Vec<_> = options
        .iter()
        .filter_map(|option| {
            let plan = option.cast_plan?;
            match plan.permission {
                CastPermission::GrantedWay { source, rule: 0 } => Some((source, option, plan)),
                _ => None,
            }
        })
        .collect();
    assert_eq!(
        routes.len(),
        9,
        "every distinct grantor must remain selectable past the former eight-way range"
    );
    let last_source = *sources.last().unwrap();
    let (_, last_option, last_plan) = routes
        .iter()
        .find(|(source, _, _)| *source == last_source)
        .expect("the ninth grantor's route is offered");
    let last_cost = cast_plan_cost(&game, PlayerId(0), spell, *last_plan).unwrap();
    assert_eq!(last_cost.mana, ManaCost::ZERO);
    assert_eq!(last_cost.life, 0);
    let first_cost = cast_plan_cost(&game, PlayerId(0), spell, routes[0].2).unwrap();
    assert_eq!(
        first_cost.life, 1,
        "the first source retains its own fixed cost"
    );

    let request = game.pending().unwrap().clone();
    game.answer(
        request.id,
        request.player,
        Answer::Act(Action::of(last_option)),
    )
    .unwrap();
    resolve_free_cast(&mut game);

    assert!(
        game.state
            .zone(Zone::exile(PlayerId(0)))
            .iter()
            .any(|&id| game.state.obj(id).card == table().find("Cast Plan Probe").unwrap()),
        "the selected grantor gave the spell flashback, so it leaves the stack for exile"
    );
    assert!(
        game.log.iter().any(|record| matches!(record.event,
        Event::AbilityActivated { source, ability, player: PlayerId(0) }
            if source == last_source && ability.index == GRANTED_WAY_USED)),
        "the history must identify the actual grantor whose route was used"
    );
}
