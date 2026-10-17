use cantrip_api::*;
use cantrip_mechanics::attach::attached_object;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "Enchant artifact or non-Aura enchantment" (CR 303.4a, 115.1): a permanent that is an artifact, or an enchantment that is not an
/// Aura, as the layers give it - checked as the Aura is cast and, being the Aura's restriction, again whenever it is on the battlefield
/// (CR 303.4d, 704.5m). The Aura subtype is no word of this card's text (Aura is an enchantment type, CR 303.4).
fn artifact_or_non_aura_enchantment(q: &dyn Query, _: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if targets::permanent(q, id) && {
        let c = q.chars(id);
        c.is(CardType::Artifact) || (c.is(CardType::Enchantment) && !c.has_subtype(Subtype::Aura))
    })
}

/// Layer 4 (CR 613.1d): "is a Construct creature ... in addition to its other types" - it keeps every type and subtype it had, and
/// gains the creature type and the creature type Construct, read through the words of this card's text (CR 612.2).
fn construct_creature_too(q: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    chars.types = chars.types.with(CardType::Creature);
    chars.subtypes = chars.subtypes.with(q.words(source).subtype(Subtype::Construct));
}

/// Layer 7b (CR 613.4b): "base power and toughness 5/5", under counters and every modifier.
fn base_five_five(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.power = Some(5);
    chars.toughness = Some(5);
}

pub static CARD: CardDef = CardDef {
    enchant: Some(target(artifact_or_non_aura_enchantment)),
    abilities: &[ability(cost("{4}{G}"), effects::return_this_to_hand, "{4}{G}: Return this card from your graveyard to your hand.").from(ZoneKind::Graveyard)],
    statics: &[
        static_ability(
            Layer::Type,
            attached_object,
            construct_creature_too,
            "Enchanted permanent is a Construct creature with base power and toughness 5/5 in addition to its other types.",
        ),
        static_ability(
            Layer::PtSet,
            attached_object,
            base_five_five,
            "Enchanted permanent is a Construct creature with base power and toughness 5/5 in addition to its other types.",
        ),
    ],
    ..oracle::PUPPET_CRAFTING
};
