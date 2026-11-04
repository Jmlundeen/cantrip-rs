use cantrip_api::*;
use cantrip_mechanics::{delayed, targets, triggers};
use cantrip_oracle as oracle;

/// "Whenever enchanted creature blocks or becomes blocked by a non-Wall creature" (CR 509.1, 509.3b, 603.2): the creature opposing
/// the enchanted creature, if it is not a Wall, the type read through the words of Venom's text (CR 612).
fn blocks_or_becomes_blocked(q: &dyn Query, asker: &Asker, event: &Event) -> Option<Fired> {
    let enchanted = q.obj(asker.source).attached_to?;
    let other = triggers::other_creature_in_block_with_this(q, enchanted, event)?;
    if q.chars(other).has_subtype(q.words(asker.source).subtype(Subtype::Wall)) {
        return None;
    }
    Fired::about(Target::Object(other))
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when_asked(EventMask::BLOCKS, blocks_or_becomes_blocked),
        delayed::destroy_that_creature_at_end_of_combat,
        "Whenever enchanted creature blocks or becomes blocked by a non-Wall creature, destroy the other creature at end of combat.",
    )],
    delayed: &[delayed::destroy_that_at_end_of_combat("Destroy the other creature.")],
    enchant: Some(targets::CREATURE),
    ..oracle::VENOM
};
