use cantrip_api::*;
use cantrip_mechanics::replacements::GAIN_TWICE_THAT_MUCH_LIFE;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef { replacements: &[GAIN_TWICE_THAT_MUCH_LIFE], ..oracle::BOON_REFLECTION };
