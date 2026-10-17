use cantrip_api::*;
use cantrip_mechanics::prevention;
use cantrip_oracle as oracle;

/// "Prevent all combat damage that would be dealt this turn" (CR 615.1, 611.2): a prevention effect until end of turn, of
/// combat damage dealt to anyone by anything. It exists only once Fog resolves (CR 615.4).
fn prevent_combat_damage(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    prevention::all_combat_damage_this_turn(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], prevent_combat_damage),
    granted_preventions: &[prevention::COMBAT_DAMAGE],
    ..oracle::FOG
};
