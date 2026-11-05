use cantrip_api::*;
use cantrip_mechanics::triggers::ward;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "Tap up to one target creature. Draw a card." (CR 701.26a, 121.1): the target is tapped only if it is still legal as the spell
/// resolves; the card is drawn either way. A spell with a target that is no longer legal does not resolve at all (CR 608.2b).
fn skimming_strike(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::tap_targets(g, item)?;
    effects::draw(g, item, 1)
}

/// The Omen (CR 720): "Tap up to one target creature. Draw a card. (Then shuffle this card into its owner's library.)" Cast from
/// the hand as an Omen instant for {1}{U}; as it resolves it is shuffled into its owner's library (CR 720.3d, the core's).
pub static SKIMMING_STRIKE: CardDef = CardDef {
    spell: spell(&[targets::CREATURE.between(0, 1)], skimming_strike),
    ..oracle::SKIMMING_STRIKE
};

pub static CARD: CardDef = CardDef {
    abilities: &[ward::<2>("Ward {2}")],
    omen: Some(&SKIMMING_STRIKE),
    ..oracle::DIRGUR_ISLAND_DRAGON
};

pub static EXTRA: &[&CardDef] = &[&SKIMMING_STRIKE];
