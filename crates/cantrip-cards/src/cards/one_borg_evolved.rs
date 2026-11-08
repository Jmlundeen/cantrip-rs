use cantrip_api::*;
use cantrip_mechanics::{statics, triggers::ward};
use cantrip_oracle as oracle;

/// "Artifact creatures you control have flying, double strike, vigilance, menace, lifelink, and ward {2}." (CR 613.1f,
/// 702.21): one static ability in layer 6 granting the five keywords and, through the card's delayed ward ability
/// (CR 113.2c, `statics::grant_delayed_ability`), its own ward {2} to each artifact creature its controller controls.
fn have_evolved_keywords(view: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    chars.keywords = chars
        .keywords
        .with(Keyword::Flying)
        .with(Keyword::DoubleStrike)
        .with(Keyword::Vigilance)
        .with(Keyword::Menace)
        .with(Keyword::Lifelink);
    statics::grant_delayed_ability(view, source, chars);
}

pub static CARD: CardDef = CardDef {
    delayed: &[ward::<2>("Ward {2} (Whenever this creature becomes the target of a spell or ability an opponent controls, counter it unless that player pays {2}.)")],
    statics: &[static_ability(
        Layer::Ability,
        statics::your_artifact_creatures,
        have_evolved_keywords,
        "Artifact creatures you control have flying, double strike, vigilance, menace, lifelink, and ward {2}.",
    )],
    ..oracle::ONE_BORG_EVOLVED
};
