use cantrip_api::*;
use cantrip_mechanics::{discard_chosen, targets, tokens};
use cantrip_oracle as oracle;

/// +1: target player draws two cards, then discards two cards (their choice, asked unless the hand is empty).
fn plus_one(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(player) = item.targets[0].player() else { return Ok(()) };
    g.draw(player, 2);
    discard_chosen(g, player, item.object, 2)?;
    Ok(())
}

/// -2: gain control of target artifact (CR 108.4), with no duration: nothing ever gives it back.
fn steal(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(artifact) = targets::object(g, item, 0) {
        g.gain_control(item.object, artifact, item.controller, Duration::Indefinite);
    }
    Ok(())
}

fn emblem(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create_emblem(g, item.controller, &EMBLEM);
    Ok(())
}

// --- The emblem ---------------------------------------------------------------------------------------------------

/// The permanents a spell targets, on the battlefield right now (CR 110.1). A spell that has left the stack (countered
/// in response) targeted what it last did (CR 608.2h).
fn permanents_targeted(q: &dyn Query, spell: ObjectId) -> Vec<ObjectId> {
    let state = q.state();
    let Some(item) = state.stack.iter().find(|i| i.kind == StackKind::Spell && i.object == spell).or_else(|| state.left_stack(spell)) else {
        return Vec::new();
    };
    item.targets[..item.target_count as usize]
        .iter()
        .filter_map(|t| t.object())
        .filter(|&id| targets::permanent(q, id))
        .collect()
}

/// "Whenever you cast a spell that targets one or more permanents": a question about the spell, whose targets are
/// public from the moment it was announced (CR 601.2c). A spell that targets only players, or only cards in a
/// graveyard, sets nothing off.
fn you_cast_a_spell_that_targets_a_permanent(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::SpellCast { spell, player, .. } if *player == q.controller(source) && !permanents_targeted(q, *spell).is_empty() => {
            Fired::about(Target::Object(*spell))
        }
        _ => None,
    }
}

/// "Gain control of those permanents": read again as the ability resolves, so one that has left in response is not
/// one to gain control of. The ability is above the spell on the stack, so they change hands before the spell does
/// anything to them, and it resolves with targets its controller now owns (CR 608.2b).
fn seize(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Target::Object(spell) = item.subject else { return Ok(()) };
    for permanent in permanents_targeted(g, spell) {
        g.gain_control(item.object, permanent, item.controller, Duration::Indefinite);
    }
    Ok(())
}

pub static EMBLEM: CardDef = CardDef {
    name: "Dack Fayden emblem",
    has_cost: false,
    abilities: &[AbilityDef::Triggered(TriggeredDef {
        text: "Whenever you cast a spell that targets one or more permanents, gain control of those permanents.",
        listens: EventMask::SPELL_CAST,
        matches: you_cast_a_spell_that_targets_a_permanent,
        effect: seize,
        ..TriggeredDef::BLANK
    })],
    text: "Whenever you cast a spell that targets one or more permanents, gain control of those permanents.",
    ..CardDef::BLANK
};

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Activated(ActivatedDef {
            text: "+1: Target player draws two cards, then discards two cards.",
            cost: Cost { loyalty: Some(1), ..Cost::NONE },
            timing: Timing::Sorcery,
            targets: &[targets::PLAYER],
            effect: plus_one,
            ..ActivatedDef::BLANK
        }),
        AbilityDef::Activated(ActivatedDef {
            text: "\u{2212}2: Gain control of target artifact.",
            cost: Cost { loyalty: Some(-2), ..Cost::NONE },
            timing: Timing::Sorcery,
            targets: &[targets::ARTIFACT],
            effect: steal,
            ..ActivatedDef::BLANK
        }),
        AbilityDef::Activated(ActivatedDef {
            text: "\u{2212}6: You get an emblem with \"Whenever you cast a spell that targets one or more permanents, gain control of those permanents.\"",
            cost: Cost { loyalty: Some(-6), ..Cost::NONE },
            timing: Timing::Sorcery,
            effect: emblem,
            ..ActivatedDef::BLANK
        }),
    ],
    ..oracle::DACK_FAYDEN
};

pub static EXTRA: &[&CardDef] = &[&EMBLEM];
