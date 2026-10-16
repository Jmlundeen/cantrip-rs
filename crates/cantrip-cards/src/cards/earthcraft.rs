use cantrip_api::*;
use cantrip_mechanics::{TAP_AN_UNTAPPED_CREATURE, targets};
use cantrip_oracle as oracle;

/// "Target basic land" (CR 115.1): a land on the battlefield with the basic supertype.
fn basic_land(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    (targets::LAND.filter)(q, asker, t) && matches!(t, Target::Object(id) if q.chars(id).supertypes.basic)
}

/// Untap target basic land.
fn untap_land(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(land) = targets::object(g, item, 0) {
        g.untap(land);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(TAP_AN_UNTAPPED_CREATURE, untap_land, "Tap an untapped creature you control: Untap target basic land.")
        .targets(&[target(basic_land)])],
    ..oracle::EARTHCRAFT
};
