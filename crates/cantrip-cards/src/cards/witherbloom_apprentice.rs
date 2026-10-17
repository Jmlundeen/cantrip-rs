use cantrip_api::*;
use cantrip_mechanics::triggers::MAGECRAFT;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(MAGECRAFT, cantrip_mechanics::effects::each_opponent_loses_1_life_you_gain_1, "Magecraft \u{2014} Whenever you cast or copy an instant or sorcery spell, each opponent loses 1 life and you gain 1 life.")],
    ..oracle::WITHERBLOOM_APPRENTICE
};
