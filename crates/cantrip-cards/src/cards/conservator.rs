use cantrip_api::*;
use cantrip_mechanics::prevention;
use cantrip_oracle as oracle;

/// Prevent the next two damage to the ability's controller this turn (CR 615.7).
fn prevent_damage(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    prevention::this_turn(g, item, Target::Player(item.controller), 0, Shield::Next(2));
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(cost("{3}").and(TAP), prevent_damage, "{3}, {T}: Prevent the next 2 damage that would be dealt to you this turn.")],
    granted_preventions: &[GrantedPreventionDef { hint: "prevents up to 2 damage", ..prevention::ANY_DAMAGE.text("Prevent the next 2 damage that would be dealt to you this turn.") }],
    ..oracle::CONSERVATOR
};
