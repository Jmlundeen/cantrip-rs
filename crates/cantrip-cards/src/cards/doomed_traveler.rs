use cantrip_api::*;
use cantrip_mechanics::tokens;
use cantrip_mechanics::triggers::THIS_DIES;
use cantrip_oracle as oracle;

/// "When this creature dies, create a 1/1 white Spirit creature token with flying." The token is made for the ability's
/// controller, who is the Traveler's controller as it died (CR 603.3a, 603.10a), not its owner.
fn spirit(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::WHITE_FLYING_SPIRIT, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_DIES, spirit, "When this creature dies, create a 1/1 white Spirit creature token with flying.").look_back()],
    ..oracle::DOOMED_TRAVELER
};

pub static EXTRA: &[&CardDef] = &[&tokens::WHITE_FLYING_SPIRIT];
