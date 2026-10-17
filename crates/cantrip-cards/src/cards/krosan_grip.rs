use cantrip_api::*;
use cantrip_mechanics::effects::destroy_first_target;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    // Split second (CR 702.61a): while it is on the stack, players can't cast spells or activate abilities that aren't mana
    // abilities; the core enforces it.
    keywords: Keywords::of(&[Keyword::SplitSecond]),
    // Destroy target artifact or enchantment (CR 701.8); nothing if it is no longer a legal target (CR 608.2b).
    spell: spell(&[targets::ARTIFACT_OR_ENCHANTMENT], destroy_first_target),
    ..oracle::KROSAN_GRIP
};
