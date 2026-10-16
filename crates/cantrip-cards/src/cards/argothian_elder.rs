use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Untap two target lands": every one that is still a legal target as the ability resolves (CR 608.2b).
fn untap_lands(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    for land in targets::legal_objects(g, item) {
        g.untap(land);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    // Exactly two target lands (CR 601.2c, 602.2b): with fewer than two lands on the battlefield the ability cannot be activated.
    abilities: &[ability(cost("{T}"), untap_lands, "{T}: Untap two target lands.").targets(&[targets::LAND.between(2, 2)])],
    ..oracle::ARGOTHIAN_ELDER
};
