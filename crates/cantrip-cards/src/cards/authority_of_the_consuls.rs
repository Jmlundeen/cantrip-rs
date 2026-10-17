use cantrip_api::*;
use cantrip_mechanics::effects::gain_life;
use cantrip_mechanics::replacements::{enter_tapped, opponents_enter_tapped};
use cantrip_mechanics::triggers::OPPONENTS_CREATURE_ENTERS;
use cantrip_oracle as oracle;

fn gain_1(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    gain_life(g, item, 1)
}

pub static CARD: CardDef = CardDef {
    replacements: &[opponents_enter_tapped::<{ enter_tapped::CREATURES }>("enters tapped")],
    abilities: &[triggered(OPPONENTS_CREATURE_ENTERS, gain_1, "Whenever a creature an opponent controls enters, you gain 1 life.")],
    ..oracle::AUTHORITY_OF_THE_CONSULS
};
