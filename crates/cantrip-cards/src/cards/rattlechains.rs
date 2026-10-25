use cantrip_api::*;
use cantrip_mechanics::{effects, targets, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// "Target Spirit": a Spirit permanent on the battlefield, its creature type read through the words of this card's text
/// (CR 612) and judged again as the ability resolves (CR 608.2b).
fn spirit(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    let Target::Object(id) = t else { return false };
    targets::permanent(q, id) && q.chars(id).has_subtype(q.words(asker.source).subtype(Subtype::Spirit))
}

/// "When this creature enters, target Spirit gains hexproof until end of turn." (CR 702.11b, 611.2a)
fn hexproof(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(spirit) = targets::object(g, item, 0) else { return Ok(()) };
    effects::gain_hexproof_until_end_of_turn(g, item.object, spirit);
    Ok(())
}

/// "You may cast Spirit spells as though they had flash" (CR 601.3, 601.3b, 702.8a): its controller's, from whatever zone
/// they cast them from, the spell judged as it would be on the stack under them (CR 601.3e: an opponent's creature card
/// their Arcane Adaptation makes a Spirit as their spell); the Spirit type is read through the words of this card's text
/// (CR 612).
fn spirit_you_cast(q: &dyn Query, asker: &Asker, spell: ObjectId, caster: PlayerId) -> bool {
    caster == asker.controller && q.chars(spell).has_subtype(q.words(asker.source).subtype(Subtype::Spirit))
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, hexproof, "When this creature enters, target Spirit gains hexproof until end of turn.")
        .targets(&[target(spirit)])],
    flash_rules: &[spirit_you_cast],
    ..oracle::RATTLECHAINS
};
