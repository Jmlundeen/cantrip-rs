use cantrip_api::*;
use cantrip_oracle as oracle;

/// "{T}: Add {U}. Spend this mana only to cast an artifact spell." (CR 106.6): the shared tap-for-restricted-mana
/// ability; Vigilance is a core keyword the content file carries, so the card adds only this line.
pub static CARD: CardDef = CardDef {
    abilities: &[mana_ability(
        TAP,
        add(Mana::U).only_for(ManaRestriction::ARTIFACT_SPELLS),
        "{T}: Add {U}. Spend this mana only to cast an artifact spell.",
    )],
    ..oracle::LA_FORGE_PERCEPTIVE_ENGINEER
};
