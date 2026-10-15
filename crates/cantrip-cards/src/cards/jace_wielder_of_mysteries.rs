use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "If you would draw a card while your library has no cards in it": one draw (CR 121.1), judged before any card is
/// moved, with nothing left to draw (CR 121.6a).
fn would_draw_from_nothing(q: &dyn Query, asker: &Asker, proposal: &Proposal) -> bool {
    matches!(proposal, Proposal::Draw(d) if d.player == asker.controller && q.state().zone(Zone::library(d.player)).is_empty())
}

/// The draw does not happen (CR 614.6).
fn not_drawn(_: &dyn Query, _: &Asker, proposal: &mut Proposal) {
    if let Proposal::Draw(d) = proposal {
        d.replaced = true;
    }
}

/// "You win the game instead."
fn win(g: &mut dyn Game, asker: &Asker, _: &Proposal, _: &Proposal) {
    g.win_game(asker.controller);
}

/// +1: "Target player mills two cards. Draw a card."
fn plus_one(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if g.target_legal(item, 0)
        && let Some(player) = item.targets[0].player()
    {
        g.mill(player, 2);
    }
    g.draw(item.controller, 1);
    Ok(())
}

/// -8: "Draw seven cards. Then if your library has no cards in it, you win the game."
fn minus_eight(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.draw(item.controller, 7);
    if g.state().zone(Zone::library(item.controller)).is_empty() {
        g.win_game(item.controller);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Activated(ActivatedDef {
            text: "+1: Target player mills two cards. Draw a card.",
            cost: Cost { loyalty: Some(1), ..Cost::NONE },
            timing: Timing::Sorcery,
            targets: &[targets::PLAYER],
            effect: plus_one,
            ..ActivatedDef::BLANK
        }),
        AbilityDef::Activated(ActivatedDef {
            text: "\u{2212}8: Draw seven cards. Then if your library has no cards in it, you win the game.",
            cost: Cost { loyalty: Some(-8), ..Cost::NONE },
            timing: Timing::Sorcery,
            effect: minus_eight,
            ..ActivatedDef::BLANK
        }),
    ],
    replacements: &[ReplacementDef {
        hint: "wins the game instead of drawing from an empty library",
        kind: ReplacementKind::Draw,
        applies: would_draw_from_nothing,
        apply: not_drawn,
        then: Some(win),
        ..ReplacementDef::BLANK
    }],
    ..oracle::JACE_WIELDER_OF_MYSTERIES
};
