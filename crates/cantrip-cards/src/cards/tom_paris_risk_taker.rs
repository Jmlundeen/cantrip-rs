use cantrip_api::*;
use cantrip_mechanics::{effects, seek_out};
use cantrip_oracle as oracle;

/// "Seek out — Whenever you draw your second card each turn, creatures you control gain flying, double strike, vigilance,
/// and haste until end of turn." (CR 611.2c, 702.9, 702.4, 702.20, 702.10): the creatures this player controls as the
/// ability resolves gain all four until the turn ends; the other seats' creatures gain nothing, and a creature that
/// arrives later is not affected.
fn gain_four_keywords(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::creatures_you_control_gain(
        g,
        item,
        &[Keyword::Flying, Keyword::DoubleStrike, Keyword::Vigilance, Keyword::Haste],
    );
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[seek_out::seek_out(
        gain_four_keywords,
        &[],
        "Seek out \u{2014} Whenever you draw your second card each turn, creatures you control gain flying, double strike, vigilance, and haste until end of turn.",
    )],
    ..oracle::TOM_PARIS_RISK_TAKER
};
