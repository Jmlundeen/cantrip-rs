use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::seek_out::seek_out;
use cantrip_oracle as oracle;

/// "Seek out — Whenever you draw your second card each turn, you gain 4 life" (CR 119.3, 603.2): the trigger is the
/// mechanic's, and the life is gained as the ability resolves.
fn gain_four(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::gain_life(g, item, 4)
}

pub static CARD: CardDef = CardDef {
    abilities: &[seek_out(gain_four, &[], "Seek out — Whenever you draw your second card each turn, you gain 4 life.")],
    ..oracle::DR_PULASKI_CYNICAL_CLINICIAN
};
