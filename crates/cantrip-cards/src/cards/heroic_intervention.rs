use cantrip_api::*;
use cantrip_mechanics::effects::gain_hexproof_and_indestructible_until_end_of_turn as protect;
use cantrip_oracle as oracle;

/// "Permanents you control gain hexproof and indestructible until end of turn." (CR 608.2, 611.2a).
fn permanents_control_gain_hexproof_indestructible(
    g: &mut dyn Game,
    item: &StackItem,
) -> Result<(), Suspend> {
    let permanents = g.permanents_of(item.controller);
    instruction(g, |g| {
        for object in permanents {
            protect(g, item.object, object);
        }
    });
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], permanents_control_gain_hexproof_indestructible),
    ..oracle::HEROIC_INTERVENTION
};
