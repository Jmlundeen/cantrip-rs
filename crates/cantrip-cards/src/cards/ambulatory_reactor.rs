use cantrip_api::*;
use cantrip_mechanics::statics;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    // "{T}: Add {C}. Spend this mana only to cast an artifact spell or activate an ability of an artifact source.",
    // the mana ability the creatures are given (CR 605.1a). The creature that has it is the ability's source
    // (CR 113.7), so each artifact creature taps for its own {C}, and the restriction is the mana's (CR 106.6).
    delayed: &[mana_ability(
        TAP,
        add(Mana::C).only_for(ManaRestriction::ARTIFACT_SPELLS_OR_ABILITIES),
        "{T}: Add {C}. Spend this mana only to cast an artifact spell or activate an ability of an artifact source.",
    )],
    // "Artifact creatures you control have ..." (CR 613.1f, layer 6): judged on what the earlier layers made each
    // permanent, so a creature that becomes an artifact creature has it, and an opponent's never does.
    statics: &[static_ability(
        Layer::Ability,
        statics::your_artifact_creatures,
        statics::grant_delayed_ability,
        "Artifact creatures you control have \"{T}: Add {C}. Spend this mana only to cast an artifact spell or activate an ability of an artifact source.\"",
    )],
    ..oracle::AMBULATORY_REACTOR
};
