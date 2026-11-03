use cantrip_api::*;
use cantrip_mechanics::{prevention, targets};
use cantrip_oracle as oracle;

/// Prevent the next damage to the targeted creature this turn (CR 615.7).
fn prevent(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    prevention::next_damage_to_target(g, item, 0, 0, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        TAP,
        prevent,
        "{T}: Prevent the next 1 damage that would be dealt to target creature this turn.",
    )
    .targets(&[targets::CREATURE])],
    granted_preventions: &[prevention::ANY_DAMAGE.text(
        "Prevent the next 1 damage that would be dealt to target creature this turn.",
    )],
    ..oracle::OASIS
};
