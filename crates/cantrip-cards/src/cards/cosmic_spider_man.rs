use cantrip_api::*;
use cantrip_mechanics::effects::other_of_subtype_you_control_gain_until_end_of_turn;
use cantrip_mechanics::lands;
use cantrip_mechanics::triggers::BEGINNING_OF_YOUR_COMBAT;
use cantrip_oracle as oracle;

/// Flying, first strike, trample, lifelink and haste are core keywords. "At the beginning of combat on your turn, other Spiders you
/// control gain flying, first strike, trample, lifelink, and haste until end of turn." (CR 611.2, 603.2)
pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        BEGINNING_OF_YOUR_COMBAT,
        |g, item| {
            other_of_subtype_you_control_gain_until_end_of_turn::<
                { lands::sub(Subtype::Spider) },
                { Keywords::of(&[Keyword::Flying, Keyword::FirstStrike, Keyword::Trample, Keyword::Lifelink, Keyword::Haste]).0 },
            >(g, item)
        },
        "At the beginning of combat on your turn, other Spiders you control gain flying, first strike, trample, lifelink, and haste until end of turn.",
    )],
    ..oracle::COSMIC_SPIDER_MAN
};
