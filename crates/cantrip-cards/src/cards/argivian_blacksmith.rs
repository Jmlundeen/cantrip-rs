use cantrip_api::*;
use cantrip_mechanics::{prevention, targets};
use cantrip_oracle as oracle;

/// Prevent the next 2 damage to the target this turn (CR 615.7).
fn prevent_damage(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    prevention::next_damage_to_target(g, item, 0, 0, 2);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        TAP,
        prevent_damage,
        "{T}: Prevent the next 2 damage that would be dealt to target artifact creature this turn.",
    ).targets(&[targets::ARTIFACT_CREATURE])],
    granted_preventions: &[prevention::ANY_DAMAGE.text("Prevent the next 2 damage that would be dealt to target artifact creature this turn.")],
    ..oracle::ARGIVIAN_BLACKSMITH
};
