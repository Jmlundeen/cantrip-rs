use cantrip_api::*;
use cantrip_mechanics::{gift, targets, tokens};
use cantrip_oracle as oracle;

/// "Target creature an opponent controls", or, if the gift was promised, "target nonland permanent an opponent controls"
/// instead. The promise is announced before the target is chosen (CR 601.2b-c, 702.174m), so the target is judged by it,
/// and again as the spell resolves (CR 608.2b).
fn creature_or_nonland_permanent(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    let filter = if gift::promised(q, asker.source) {
        targets::NONLAND_PERMANENT_AN_OPPONENT_CONTROLS
    } else {
        targets::CREATURE_AN_OPPONENT_CONTROLS
    };
    (filter.filter)(q, asker, t)
}

/// "Return target creature an opponent controls to its owner's hand. If the gift was promised, instead return target
/// nonland permanent an opponent controls to its owner's hand." The gift itself was given before this runs
/// (CR 702.174j).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(permanent) = targets::object(g, item, 0) {
        g.return_to_hand(&[permanent]);
    }
    Ok(())
}

/// "Gift a tapped Fish" (CR 702.174f): the Fish is the card's own token.
pub static EXTRA: &[&CardDef] = &[&tokens::FISH];

pub static CARD: CardDef = CardDef {
    spell: spell(&[target(creature_or_nonland_permanent)], effect),
    gift: Some(gift::TAPPED_FISH),
    ..oracle::INTO_THE_FLOOD_MAW
};
