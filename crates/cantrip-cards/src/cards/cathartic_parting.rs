use cantrip_api::*;
use cantrip_mechanics::{targets, effects};
use cantrip_oracle as oracle;

/// "Target artifact or enchantment an opponent controls" (CR 115.1).
fn opponents_artifact_or_enchantment(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    (targets::ARTIFACT_OR_ENCHANTMENT.filter)(q, asker, t) && matches!(t, Target::Object(id) if q.controller(id) != asker.controller)
}

/// "Target cards from your graveyard" (CR 115.1).
fn card_in_your_graveyard(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    targets::card_in_your_graveyard(q, asker, t, &[])
}

/// The owner of the target permanent shuffles it into their library, then you may shuffle up to four target cards
/// from your graveyard into your library. A permanent that is no longer a legal target does nothing and has no owner
/// to shuffle (CR 608.2b); the second sentence is a "you may" asked as it resolves, and a yes shuffles the caster's
/// library even when no card of it remains a legal target (CR 701.24c-d).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(permanent) = targets::object(g, item, 0) {
        g.put_into_library(&[permanent], LibraryPlace::ShuffledIn);
    }

    let you = item.controller;
    let cards: Vec<ObjectId> = (1..item.target_count as usize).filter_map(|slot| targets::object(g, item, slot)).collect();
    if !effects::ask_may(g, you, item.object, Purpose::Optional)? {
        return Ok(());
    }
    g.put_into_library(&cards, LibraryPlace::Top);
    g.shuffle(you);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef {
        targets: &[
            TargetSpec { min: 1, max: 1, filter: opponents_artifact_or_enchantment, label: "" },
            TargetSpec { min: 0, max: 4, filter: card_in_your_graveyard, label: "" },
        ],
        effect,
    }),
    ..oracle::CATHARTIC_PARTING
};
