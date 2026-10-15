use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Each opponent can't draw more than one card each turn." A draw instruction is cut down to the one card the player
/// has not drawn yet this turn (CR 101.2, 121.2): nothing at all once they have drawn.
fn limits(_: &dyn Query, asker: &Asker, proposal: &Proposal) -> bool {
    matches!(proposal, Proposal::DrawInstruction(d) if d.player != asker.controller)
}

fn limit(q: &dyn Query, _: &Asker, proposal: &mut Proposal) {
    if let Proposal::DrawInstruction(d) = proposal {
        let allowed = 1u32.saturating_sub(cantrip_mechanics::history::drawn_this_turn(q, d.player) as u32);
        d.count = d.count.min(allowed);
    }
}

/// "Whenever you or a permanent you control becomes the target of a spell or ability an opponent controls": a
/// permanent is one on the battlefield. Each thing named is its own event, so each triggers it (CR 603.2).
fn targeted(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    let you = q.controller(source);
    match event {
        Event::BecameTarget { target, controller, .. } if *controller != you => {
            let yours = match target {
                Target::Player(p) => *p == you,
                Target::Object(id) => targets::permanent(q, *id) && q.controller(*id) == you,
                _ => false,
            };
            if yours { Fired::PLAIN } else { None }
        }
        _ => None,
    }
}

/// "You may draw a card." Not offered when an opponent's Leovold has already used up this turn's draw (CR 121.3).
fn may_draw(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let leovold = g.chars(item.object).card;
    let barred = cantrip_mechanics::history::drawn_this_turn(g, you) >= 1
        && g.state().battlefield().iter().any(|&id| g.controller(id) != you && g.chars(id).card == leovold);
    if barred {
        return Ok(());
    }
    if cantrip_mechanics::effects::ask_may(g, you, item.object, Purpose::Optional)? {
        g.draw(you, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[AbilityDef::Triggered(TriggeredDef {
        text: "Whenever you or a permanent you control becomes the target of a spell or ability an opponent controls, you may draw a card.",
        listens: EventMask::BECAME_TARGET,
        matches: targeted,
        effect: may_draw,
        ..TriggeredDef::BLANK
    })],
    replacements: &[ReplacementDef {
        hint: "can't draw more than one card each turn",
        kind: ReplacementKind::DrawInstruction,
        applies: limits,
        apply: limit,
        ..ReplacementDef::BLANK
    }],
    ..oracle::LEOVOLD_EMISSARY_OF_TREST
};
