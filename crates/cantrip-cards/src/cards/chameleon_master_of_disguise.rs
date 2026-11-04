use cantrip_api::*;
use cantrip_mechanics::{copy, mayhem};
use cantrip_oracle as oracle;

/// "You may have Chameleon enter as a copy of a creature you control, except his name is Chameleon, Master of Disguise."
/// The name is part of the copy's own copiable values (CR 707.9b), so a later copy of him has it too. Only creatures its
/// controller controls are offered (CR 614.1c, 707.2).
fn enter_as_copy(g: &mut dyn Game, entry: &mut Entering) -> Result<(), Suspend> {
    copy::enter_as_copy_of_creature_you_control(g, entry, |_, copy| copy.named("Chameleon, Master of Disguise"))
}

pub static CARD: CardDef = CardDef {
    enters_as_copy: Some(enter_as_copy),
    ways: &[mayhem::way(mana("{2}{U}"), "Mayhem {2}{U}")],
    ..oracle::CHAMELEON_MASTER_OF_DISGUISE
};
