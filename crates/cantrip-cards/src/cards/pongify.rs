use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// The 3/3 green Ape creature token it makes: no other printed card makes it exactly, so it lives here rather than in
/// the shared tokens table.
pub static APE: CardDef = CardDef {
    colors: Some(ColorSet::of(Color::Green)),
    types: TypeSet::of(&[CardType::Creature]),
    power: Some(3),
    toughness: Some(3),
    ..token!(Ape)
};

/// "Destroy target creature. It can't be regenerated. Its controller creates a 3/3 green Ape creature token."
fn pongify(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::destroy_no_regen_and_its_controller_creates(g, item, &APE)
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE], pongify),
    ..oracle::PONGIFY
};

pub static EXTRA: &[&CardDef] = &[&APE];
