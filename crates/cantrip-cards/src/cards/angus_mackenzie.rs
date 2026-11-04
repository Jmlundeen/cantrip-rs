use cantrip_api::*;
use cantrip_mechanics::{combat, prevention};
use cantrip_oracle as oracle;

/// "Prevent all combat damage that would be dealt this turn." (CR 615.1, 611.2a).
fn prevent_combat_damage(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    prevention::all_combat_damage_this_turn(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(cost("{G}{W}{U}").and(TAP), prevent_combat_damage,
        "{G}{W}{U}, {T}: Prevent all combat damage that would be dealt this turn. Activate only before the combat damage step.")
        .condition(combat::only_before_combat_damage_step)],
    granted_preventions: &[prevention::COMBAT_DAMAGE.text("Prevent all combat damage that would be dealt this turn.")],
    ..oracle::ANGUS_MACKENZIE
};
