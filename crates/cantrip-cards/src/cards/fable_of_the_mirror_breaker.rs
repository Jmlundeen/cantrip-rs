use cantrip_api::*;
use cantrip_mechanics::choose;
use cantrip_mechanics::targets;
use cantrip_mechanics::tokens::{TREASURE, create_treasure};
use cantrip_mechanics::exile_and_return_transformed;
use cantrip_oracle as oracle;

fn make_treasure(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    create_treasure(g, item.controller)
}

/// The 2/2 red Goblin Shaman token chapter I makes.
pub static GOBLIN_SHAMAN: CardDef = CardDef {
    name: "Goblin Shaman",
    has_cost: false,
    colors: Some(ColorSet::of(Color::Red)),
    types: TypeSet::of(&[CardType::Creature]),
    subtypes: SubtypeSet::of(&[Subtype::Goblin, Subtype::Shaman]),
    power: Some(2),
    toughness: Some(2),
    abilities: &[AbilityDef::Triggered(TriggeredDef {
        text: "Whenever this token attacks, create a Treasure token.",
        listens: EventMask::ATTACKS,
        matches: cantrip_mechanics::triggers::this_attacks,
        effect: make_treasure,
        ..TriggeredDef::BLANK
    })],
    text: "Whenever this token attacks, create a Treasure token.",
    ..CardDef::BLANK
};

fn chapter_one(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    cantrip_mechanics::tokens::create(g, item.controller, &GOBLIN_SHAMAN, 1)?;
    Ok(())
}

/// "You may discard up to two cards. If you do, draw that many cards." Asked whenever there is a card to discard;
/// choosing none discards and draws nothing.
fn chapter_two(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let player = item.controller;
    let hand = g.state().zone(Zone::hand(player)).to_vec();
    let chosen = choose::cards(g, player, item.object, Purpose::Discard, hand, 2, true)?;
    g.discard_together(&chosen);
    if !chosen.is_empty() {
        g.draw(player, chosen.len() as u32);
    }
    Ok(())
}

fn chapter_three(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    exile_and_return_transformed(g, item.object, item.controller)
}

// --- Reflection of Kiki-Jiki ------------------------------------------------------------------------------------

/// "Another target nonlegendary creature you control."
fn another_nonlegendary_creature_you_control(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if id != asker.source && targets::permanent(q, id) && q.controller(id) == asker.controller && {
        let c = q.chars(id);
        c.is(CardType::Creature) && !c.supertypes.legendary
    })
}

/// "Create a token that's a copy of another target nonlegendary creature you control, except it has haste. Sacrifice
/// it at the beginning of the next end step."
fn copy_creature(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    let values = g.copiable_values(creature).with_keyword(Keyword::Haste);
    let me = g.card_id(&BACK);
    for token in g.create_token_copies_of(values, item.controller, 1)? {
        g.delayed_trigger(
            DelayedWhen::Step { step: Step::End, on_turn_of: None },
            item.object,
            item.controller,
            AbilityRef::delayed(me, 0),
            Target::Object(token),
        );
    }
    Ok(())
}

pub static BACK: CardDef = CardDef {
    abilities: &[AbilityDef::Activated(ActivatedDef {
        text: "{1}, {T}: Create a token that's a copy of another target nonlegendary creature you control, except it has haste. Sacrifice it at the beginning of the next end step.",
        cost: Cost { mana: mana(1, 0, 0, 0, 0, 0), tap: true, ..Cost::NONE },
        targets: &[TargetSpec { min: 1, max: 1, filter: another_nonlegendary_creature_you_control }],
        effect: copy_creature,
        ..ActivatedDef::BLANK
    })],
    delayed: &[AbilityDef::Triggered(TriggeredDef {
        text: "Sacrifice it at the beginning of the next end step.",
        effect: cantrip_mechanics::delayed::sacrifice_subject,
        ..TriggeredDef::BLANK
    })],
    ..oracle::REFLECTION_OF_KIKI_JIKI
};

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Triggered(TriggeredDef {
            text: "I \u{2014} Create a 2/2 red Goblin Shaman creature token with \"Whenever this token attacks, create a Treasure token.\"",
            chapter: 1,
            effect: chapter_one,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Triggered(TriggeredDef {
            text: "II \u{2014} You may discard up to two cards. If you do, draw that many cards.",
            chapter: 2,
            effect: chapter_two,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Triggered(TriggeredDef {
            text: "III \u{2014} Exile this Saga, then return it to the battlefield transformed under your control.",
            chapter: 3,
            effect: chapter_three,
            ..TriggeredDef::BLANK
        }),
    ],
    back: Some(&BACK),
    ..oracle::FABLE_OF_THE_MIRROR_BREAKER
};

pub static EXTRA: &[&CardDef] = &[&GOBLIN_SHAMAN, &TREASURE];
