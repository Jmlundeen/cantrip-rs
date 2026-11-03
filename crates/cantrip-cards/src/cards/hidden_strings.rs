use cantrip_api::*;
use cantrip_mechanics::{choose, cipher, targets};
use cantrip_oracle as oracle;

/// "Another target permanent" (CR 115.1, 115.3): any permanent but the one chosen for the first target.
fn another_permanent(q: &dyn Query, _: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if targets::permanent(q, id) && !q.chosen_targets().contains(&Target::Object(id)))
}

/// "You may tap or untap target permanent, then you may tap or untap another target permanent. Cipher" (CR 608.2c-d, 701.26a,
/// 702.99a): each target's controller-of-the-spell choice is made as the spell resolves, a target gone being skipped (CR 608.2b).
fn tap_or_untap_two_permanents(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let card = g.card_id(&CARD);
    choose::may_tap_or_untap_target_in_slot(g, item, card, 0)?;
    g.end_sentence();
    choose::may_tap_or_untap_target_in_slot(g, item, card, 1)?;
    g.end_sentence();
    cipher::encode(g, item)
}

pub static CARD: CardDef = CardDef {
    spell: spell(
        &[targets::PERMANENT, target(another_permanent)],
        tap_or_untap_two_permanents,
    ),
    statics: &[cipher::STATIC],
    delayed: &[cipher::TRIGGER],
    labels: choose::TAP_OR_UNTAP,
    ..oracle::HIDDEN_STRINGS
};
