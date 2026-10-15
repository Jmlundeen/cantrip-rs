use cantrip_api::*;
use cantrip_mechanics::{choose, delayed, exile_target, targets, tokens};
use cantrip_oracle as oracle;

/// +1: draw a card, then "at the beginning of the next end step, untap up to two lands" (CR 603.7), created as the ability
/// resolves.
fn plus_one(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.draw(item.controller, 1);
    delayed::at_next_end_step(g, item, 0, Target::None);
    Ok(())
}

/// Untap up to two lands, chosen as the delayed ability resolves and not targeted: any lands on the battlefield, tapped or
/// not, nothing asked when there are none.
fn untap_two_lands(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let lands: Vec<ObjectId> = g.state().battlefield().iter().copied().filter(|&id| g.chars(id).is(CardType::Land)).collect();
    for land in choose::cards(g, item.controller, item.object, Purpose::Untap, lands, 2, true)? {
        g.untap(land);
    }
    Ok(())
}

/// -3: put target nonland permanent into its owner's library third from the top (a token simply ceases to exist).
fn minus_three(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(id) = targets::object(g, item, 0) {
        g.put_into_library(&[id], LibraryPlace::FromTop(2));
    }
    Ok(())
}

fn emblem(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create_emblem(g, item.controller, &EMBLEM);
    Ok(())
}

// --- The emblem ---------------------------------------------------------------------------------------------------

fn you_draw(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::Drew { player, .. } if *player == q.controller(source) => Fired::PLAIN,
        _ => None,
    }
}

fn opponents_permanent(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if is_live(q, id) && q.is_on_battlefield(id) && q.controller(id) != asker.controller)
}

fn exile_it(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    exile_target(g, item, 0);
    Ok(())
}

pub static EMBLEM: CardDef = CardDef {
    name: "Teferi, Hero of Dominaria emblem",
    has_cost: false,
    abilities: &[AbilityDef::Triggered(TriggeredDef {
        text: "Whenever you draw a card, exile target permanent an opponent controls.",
        listens: EventMask::DREW,
        matches: you_draw,
        targets: &[TargetSpec { min: 1, max: 1, filter: opponents_permanent }],
        effect: exile_it,
        ..TriggeredDef::BLANK
    })],
    text: "Whenever you draw a card, exile target permanent an opponent controls.",
    ..CardDef::BLANK
};

pub static EXTRA: &[&CardDef] = &[&EMBLEM];

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Activated(ActivatedDef {
            text: "+1: Draw a card. At the beginning of the next end step, untap up to two lands.",
            cost: Cost { loyalty: Some(1), ..Cost::NONE },
            timing: Timing::Sorcery,
            effect: plus_one,
            ..ActivatedDef::BLANK
        }),
        AbilityDef::Activated(ActivatedDef {
            text: "\u{2212}3: Put target nonland permanent into its owner's library third from the top.",
            cost: Cost { loyalty: Some(-3), ..Cost::NONE },
            timing: Timing::Sorcery,
            targets: &[targets::NONLAND_PERMANENT],
            effect: minus_three,
            ..ActivatedDef::BLANK
        }),
        AbilityDef::Activated(ActivatedDef {
            text: "\u{2212}8: You get an emblem with \"Whenever you draw a card, exile target permanent an opponent controls.\"",
            cost: Cost { loyalty: Some(-8), ..Cost::NONE },
            timing: Timing::Sorcery,
            effect: emblem,
            ..ActivatedDef::BLANK
        }),
    ],
    delayed: &[AbilityDef::Triggered(TriggeredDef {
        text: "Untap up to two lands.",
        effect: untap_two_lands,
        ..TriggeredDef::BLANK
    })],
    ..oracle::TEFERI_HERO_OF_DOMINARIA
};
