use cantrip_api::*;
use cantrip_mechanics::look_for_noncreature_nonland;
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

/// -2: Look at the top four cards of your library. You may reveal a noncreature, nonland card from among them and put
/// it into your hand. Put the rest on the bottom of your library in a random order. With no such card among them
/// nobody is asked.
fn look(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    look_for_noncreature_nonland(g, item.controller, item.object, 4)
}

pub static CARD: CardDef = CardDef {
    abilities: &[AbilityDef::Activated(ActivatedDef {
        text: "\u{2212}2: Look at the top four cards of your library. You may reveal a noncreature, nonland card from among them and put it into your hand. Put the rest on the bottom of your library in a random order.",
        cost: Cost { loyalty: Some(-2), ..Cost::NONE },
        timing: Timing::Sorcery,
        effect: look,
        ..ActivatedDef::BLANK
    })],
    replacements: &[ReplacementDef {
        hint: "can't draw more than one card each turn",
        kind: ReplacementKind::DrawInstruction,
        applies: limits,
        apply: limit,
        ..ReplacementDef::BLANK
    }],
    ..oracle::NARSET_PARTER_OF_VEILS
};
