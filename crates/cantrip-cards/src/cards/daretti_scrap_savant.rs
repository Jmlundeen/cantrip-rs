use cantrip_api::*;
use cantrip_mechanics::{choose, sacrifice_chosen, targets, tokens};
use cantrip_oracle as oracle;

/// +2: "Discard up to two cards, then draw that many cards." The cards are discarded together (CR 701.9) and as many
/// are drawn as were discarded, so choosing none draws none.
fn plus_two(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let hand = g.state().zone(Zone::hand(item.controller)).to_vec();
    let answer = choose::cards(g, item.controller, item.object, Purpose::Discard, hand.clone(), 2, true)?;
    let chosen: Vec<ObjectId> = hand.into_iter().filter(|c| answer.contains(c)).collect();
    if chosen.is_empty() {
        return Ok(());
    }
    g.discard_together(&chosen);
    g.draw(item.controller, chosen.len() as u32);
    Ok(())
}

fn artifact_card_in_your_graveyard(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    targets::card_in_your_graveyard(q, asker, t, &[CardType::Artifact])
}

fn is_artifact(c: &Chars) -> bool {
    c.is(CardType::Artifact)
}

/// -2: "Sacrifice an artifact. If you do, return target artifact card from your graveyard to the battlefield." The
/// artifact is chosen as the ability resolves (CR 701.21a); the return happens only if one was sacrificed
/// (CR 608.2c), and only to a target still legal (CR 608.2b).
fn minus_two(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    if !g.permanents_of(you).into_iter().any(|id| is_artifact(&g.chars(id))) {
        return Ok(());
    }
    sacrifice_chosen(g, you, item.object, 1, is_artifact)?;
    let Some(card) = targets::object(g, item, 0) else { return Ok(()) };
    g.put_onto_battlefield(card, you, false)?;
    Ok(())
}

fn make_emblem(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create_emblem(g, item.controller, &EMBLEM);
    Ok(())
}

// --- The emblem ---------------------------------------------------------------------------------------------------

/// "Whenever an artifact is put into your graveyard from the battlefield": your graveyard is the card owner's, and it
/// was an artifact as it last was on the battlefield (CR 603.10a).
fn artifact_to_your_graveyard(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { old, new, from, to, .. }
            if from.is(ZoneKind::Battlefield) && *to == Zone::graveyard(q.controller(source)) && q.chars(*old).is(CardType::Artifact) =>
        {
            Fired::about(Target::Object(*new))
        }
        _ => None,
    }
}

/// Set up "at the beginning of the next end step, return that card to the battlefield" (CR 603.7), about the card in
/// the graveyard. Created even when the card has already left the graveyard (CR 603.7c).
fn schedule_return(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    cantrip_mechanics::delayed::at_next_end_step(g, item, 0, item.subject);
    Ok(())
}

pub static EMBLEM: CardDef = CardDef {
    name: "Daretti, Scrap Savant emblem",
    has_cost: false,
    abilities: &[AbilityDef::Triggered(TriggeredDef {
        text: "Whenever an artifact is put into your graveyard from the battlefield, return that card to the battlefield at the beginning of the next end step.",
        listens: EventMask::ZONE_CHANGE,
        matches: artifact_to_your_graveyard,
        effect: schedule_return,
        ..TriggeredDef::BLANK
    })],
    delayed: &[AbilityDef::Triggered(TriggeredDef { text: "Return that card to the battlefield.", effect: cantrip_mechanics::delayed::return_subject_to_battlefield, ..TriggeredDef::BLANK })],
    text: "Whenever an artifact is put into your graveyard from the battlefield, return that card to the battlefield at the beginning of the next end step.",
    ..CardDef::BLANK
};

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Activated(ActivatedDef {
            text: "+2: Discard up to two cards, then draw that many cards.",
            cost: Cost { loyalty: Some(2), ..Cost::NONE },
            timing: Timing::Sorcery,
            effect: plus_two,
            ..ActivatedDef::BLANK
        }),
        AbilityDef::Activated(ActivatedDef {
            text: "\u{2212}2: Sacrifice an artifact. If you do, return target artifact card from your graveyard to the battlefield.",
            cost: Cost { loyalty: Some(-2), ..Cost::NONE },
            timing: Timing::Sorcery,
            targets: &[TargetSpec { min: 1, max: 1, filter: artifact_card_in_your_graveyard }],
            effect: minus_two,
            ..ActivatedDef::BLANK
        }),
        AbilityDef::Activated(ActivatedDef {
            text: "\u{2212}10: You get an emblem with \"Whenever an artifact is put into your graveyard from the battlefield, return that card to the battlefield at the beginning of the next end step.\"",
            cost: Cost { loyalty: Some(-10), ..Cost::NONE },
            timing: Timing::Sorcery,
            effect: make_emblem,
            ..ActivatedDef::BLANK
        }),
    ],
    ..oracle::DARETTI_SCRAP_SAVANT
};

pub static EXTRA: &[&CardDef] = &[&EMBLEM];
