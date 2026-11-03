use cantrip_api::*;
use cantrip_mechanics::prevention;
use cantrip_oracle as oracle;

/// Prevent all combat damage that would be dealt this turn (CR 615.1, 611.2a).
fn prevent_combat_damage(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    prevention::all_combat_damage_this_turn(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], prevent_combat_damage),
    granted_preventions: &[GrantedPreventionDef {
        hint: "Prevent all combat damage that would be dealt this turn.",
        damage: "damage",
        ..prevention::COMBAT_DAMAGE.text("Prevent all combat damage that would be dealt this turn.")
    }],
    ..oracle::HOLY_DAY
};
