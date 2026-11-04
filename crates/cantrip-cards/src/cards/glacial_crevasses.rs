use cantrip_api::*;
use cantrip_mechanics::{lands, prevention};
use cantrip_oracle as oracle;

fn prevent_combat_damage(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    prevention::all_combat_damage_this_turn(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        sacrifice(1, lands::snow_mountain),
        prevent_combat_damage,
        "Sacrifice a snow Mountain: Prevent all combat damage that would be dealt this turn.",
    )],
    granted_preventions: &[prevention::COMBAT_DAMAGE.text("Prevent all combat damage that would be dealt this turn.")],
    ..oracle::GLACIAL_CREVASSES
};
