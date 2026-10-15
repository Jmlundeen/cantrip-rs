use cantrip_api::*;
use cantrip_mechanics::{history::first_draw_in_own_draw_step, tokens};
use cantrip_oracle as oracle;

/// "If an opponent would draw a card except the first one they draw in each of their draw steps": an opponent's draw
/// instruction (CR 121.2a), less the one card their own draw step's first draw is.
fn replaces(_: &dyn Query, asker: &Asker, proposal: &Proposal) -> bool {
    matches!(proposal, Proposal::DrawInstruction(d) if d.player != asker.controller && d.count > 0)
}

/// The cards drawn after the exempt first one are not drawn (CR 614.6).
fn not_drawn(q: &dyn Query, _: &Asker, proposal: &mut Proposal) {
    if let Proposal::DrawInstruction(d) = proposal {
        d.count = d.count.min(first_draw_in_own_draw_step(q, d.player) as u32);
    }
}

/// "Instead you create a Treasure token": one for each draw that did not happen.
fn treasures(g: &mut dyn Game, asker: &Asker, before: &Proposal, after: &Proposal) {
    if let (Proposal::DrawInstruction(b), Proposal::DrawInstruction(a)) = (before, after) {
        for _ in a.count..b.count {
            let _ = tokens::create_treasure(g, asker.controller);
        }
    }
}

pub static CARD: CardDef = CardDef {
    replacements: &[ReplacementDef {
        hint: "replaces an opponent's draw with a Treasure token",
        kind: ReplacementKind::DrawInstruction,
        applies: replaces,
        apply: not_drawn,
        then: Some(treasures),
        ..ReplacementDef::BLANK
    }],
    ..oracle::HULLBREACHER
};

pub static EXTRA: &[&CardDef] = &[&tokens::TREASURE];
