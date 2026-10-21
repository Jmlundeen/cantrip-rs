use cantrip_api::*;
use cantrip_oracle as oracle;

/// "You may cast Dragon spells without paying their mana costs": an alternative cost a static ability gives (CR 118.9, 601.2b)
/// to the Dragon spells its controller casts, from wherever they cast them, judged on the face being cast. "Dragon" is a word
/// of this card's rules text, so it is read through its words (CR 612). It is the one alternative cost of that cast (CR 118.9a);
/// additional costs, and what increases the cost, are still paid (CR 118.9d, 601.2f); X is 0 (CR 107.3b).
fn dragon_spells_you_cast(q: &dyn Query, asker: &Asker, spell: ObjectId, player: PlayerId) -> bool {
    player == asker.controller && q.chars(spell).has_subtype(q.words(asker.source).subtype(Subtype::Dragon))
}

pub static CARD: CardDef = CardDef {
    spell_alternatives: &[SpellAlternative { text: "Without paying its mana cost", applies: dragon_spells_you_cast, cost: InsteadOfManaCost::WithoutPaying, flash: false }],
    ..oracle::DRACOGENESIS
};
