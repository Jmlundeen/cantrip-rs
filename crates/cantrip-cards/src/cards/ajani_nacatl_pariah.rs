use cantrip_api::*;
use cantrip_mechanics::{choose, damage_target, exile_and_return_transformed, targets, triggers, effects, tokens};
use cantrip_oracle as oracle;

/// The 2/1 white Cat Warrior token both faces make.
pub static CAT_WARRIOR: CardDef = CardDef {
    name: "Cat Warrior",
    has_cost: false,
    colors: Some(ColorSet::of(Color::White)),
    types: TypeSet::of(&[CardType::Creature]),
    subtypes: SubtypeSet::of(&[Subtype::Cat, Subtype::Warrior]),
    power: Some(2),
    toughness: Some(1),
    ..CardDef::BLANK
};

fn create_cat(g: &mut dyn Game, controller: PlayerId) -> Result<bool, Suspend> {
    Ok(!tokens::create(g, controller, &CAT_WARRIOR, 1)?.is_empty())
}

fn enters_make_cat(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    create_cat(g, item.controller)?;
    Ok(())
}

/// "Whenever one or more other Cats you control die" (CR 603.2c): the Cats are judged as they were on the
/// battlefield, and the ability, being `once_per_batch`, triggers once for those that died together.
fn cats_die(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { old, from, to, .. }
            if *old != source
                && from.is(ZoneKind::Battlefield)
                && to.is(ZoneKind::Graveyard)
                && q.obj(*old).controller == q.obj(source).controller
                && q.chars(*old).has_subtype(q.words(source).subtype(Subtype::Cat)) =>
        {
            Fired::about(Target::Object(*old))
        }
        _ => None,
    }
}

/// "You may exile Ajani, then return him to the battlefield transformed under his owner's control." The choice is
/// asked whether or not he is still there; if he is not, there is nothing to exile (CR 400.7).
fn exile_and_return(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if !effects::ask_may(g, item.controller, item.object, Purpose::Optional)? {
        return Ok(());
    }
    let owner = g.obj(item.object).owner;
    exile_and_return_transformed(g, item.object, owner)
}

// --- Ajani, Nacatl Avenger ---------------------------------------------------------------------------------------

fn plus_two(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    for id in g.permanents_of(item.controller) {
        if g.chars(id).has_subtype(g.words(item.object).subtype(Subtype::Cat)) {
            g.add_counters(id, CounterKind::PlusOne, 1);
        }
    }
    Ok(())
}

/// "If you control a red permanent other than Ajani."
fn controls_other_red(q: &dyn Query, asker: &Asker) -> bool {
    q.permanents_of(asker.controller).into_iter().any(|id| id != asker.source && q.chars(id).colors.has(q.words(asker.source).color(Color::Red)))
}

/// "0: Create a 2/1 white Cat Warrior creature token. When you do, if you control a red permanent other than Ajani,
/// he deals damage equal to the number of creatures you control to any target." The reflexive trigger exists only if
/// the token was created (CR 603.12), and, like any triggered ability, only triggers if its condition holds (CR 603.4), checked again as it resolves.
fn zero(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let created = create_cat(g, item.controller)?;
    if created {
        let me = g.card_id(&BACK);
        g.reflexive_trigger(item.object, item.controller, AbilityRef::delayed(me, 0), Target::None);
    }
    Ok(())
}

fn damage_for_creatures(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let creatures = g.permanents_of(item.controller).into_iter().filter(|&id| g.chars(id).is(CardType::Creature)).count();
    damage_target(g, item, 0, creatures as u16);
    Ok(())
}

/// "-4: Each opponent chooses an artifact, a creature, an enchantment, and a planeswalker from among the nonland
/// permanents they control, then sacrifices the rest." Each opponent in turn order keeps one of each kind they have
/// more than one of (asked kind by kind), and what is not kept is sacrificed together.
fn minus_four(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let mut doomed = Vec::new();
    for who in g.state().apnap().collect::<Vec<_>>() {
        if who == item.controller {
            continue;
        }
        let theirs: Vec<ObjectId> = g.permanents_of(who).into_iter().filter(|&id| !g.chars(id).is(CardType::Land)).collect();
        let mut kept: Vec<ObjectId> = Vec::new();
        for kind in [CardType::Artifact, CardType::Creature, CardType::Enchantment, CardType::Planeswalker] {
            let eligible: Vec<ObjectId> = theirs.iter().copied().filter(|&id| g.chars(id).is(kind)).collect();
            if eligible.len() <= 1 {
                kept.extend(eligible);
                continue;
            }
            kept.extend(choose::card_from(g, who, item.object, Purpose::Keep, theirs.clone(), eligible)?);
        }
        doomed.extend(theirs.into_iter().filter(|id| !kept.contains(id)));
    }
    g.sacrifice(&doomed);
    Ok(())
}

pub static BACK: CardDef = CardDef {
    abilities: &[
        AbilityDef::Activated(ActivatedDef {
            text: "+2: Put a +1/+1 counter on each Cat you control.",
            cost: Cost { loyalty: Some(2), ..Cost::NONE },
            timing: Timing::Sorcery,
            effect: plus_two,
            ..ActivatedDef::BLANK
        }),
        AbilityDef::Activated(ActivatedDef {
            text: "0: Create a 2/1 white Cat Warrior creature token. When you do, if you control a red permanent other than Ajani, he deals damage equal to the number of creatures you control to any target.",
            cost: Cost { loyalty: Some(0), ..Cost::NONE },
            timing: Timing::Sorcery,
            effect: zero,
            ..ActivatedDef::BLANK
        }),
        AbilityDef::Activated(ActivatedDef {
            text: "\u{2212}4: Each opponent chooses an artifact, a creature, an enchantment, and a planeswalker from among the nonland permanents they control, then sacrifices the rest.",
            cost: Cost { loyalty: Some(-4), ..Cost::NONE },
            timing: Timing::Sorcery,
            effect: minus_four,
            ..ActivatedDef::BLANK
        }),
    ],
    delayed: &[AbilityDef::Triggered(TriggeredDef {
        text: "When you do, if you control a red permanent other than Ajani, he deals damage equal to the number of creatures you control to any target.",
        targets: &[targets::ANY],
        effect: damage_for_creatures,
        condition: Some(controls_other_red),
        ..TriggeredDef::BLANK
    })],
    ..oracle::AJANI_NACATL_AVENGER
};

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Triggered(TriggeredDef {
            text: "When Ajani enters, create a 2/1 white Cat Warrior creature token.",
            listens: EventMask::ZONE_CHANGE,
            matches: triggers::this_enters,
            effect: enters_make_cat,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Triggered(TriggeredDef {
            text: "Whenever one or more other Cats you control die, you may exile Ajani, then return him to the battlefield transformed under his owner's control.",
            listens: EventMask::ZONE_CHANGE,
            matches: cats_die,
            effect: exile_and_return,
            look_back: true,
            once_per_batch: true,
            ..TriggeredDef::BLANK
        }),
    ],
    back: Some(&BACK),
    ..oracle::AJANI_NACATL_PARIAH
};

pub static EXTRA: &[&CardDef] = &[&CAT_WARRIOR];
