use cantrip_api::*;
use cantrip_mechanics::{exile_target_until_leaves_effect, targets, triggers::THIS_ENTERS, warp};
use cantrip_oracle as oracle;

/// "Target non-Assassin creature" (CR 115.1, 205.3): a creature on the battlefield that does not have the Assassin subtype, judged
/// again as the ability resolves (CR 608.2b). "Assassin" is a word of the card's text (CR 612), read through the words the Stalker has now.
fn non_assassin_creature(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if targets::permanent(q, id) && q.chars(id).is(CardType::Creature) && !q.chars(id).has_subtype(q.words(asker.source).subtype(Subtype::Assassin)))
}

/// Warp is a keyword: `warp::way`, with its end-step exile and recast (CR 702.185).
pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        THIS_ENTERS,
        exile_target_until_leaves_effect,
        "When this creature enters, exile up to one target non-Assassin creature until this creature leaves the battlefield.",
    )
    .targets(&[up_to_one(non_assassin_creature)])],
    ways: &[warp::way(mana("{1}{W}"), "Warp {1}{W}")],
    end_step_riders: warp::RIDERS,
    delayed: &[warp::EXILE],
    ..oracle::ALL_FATES_STALKER
};
