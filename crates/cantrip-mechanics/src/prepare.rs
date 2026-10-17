//! Prepare (CR 722): a permanent with a prepare spell that is prepared lets its controller cast a copy of that spell
//! (`CardDef::prepare`); `Game::become_prepared` makes it prepared later, and this is "enters prepared".
//! Carnivorous Cultivator, Emeritus of Abundance, Emeritus of Ideation, Skycoach Conductor and Pompous Battlemage print it.

use cantrip_api::*;

/// "This creature enters prepared" (CR 614.1d, 722.3a): a replacement effect of what it is as it enters, for `CardDef::enters`.
pub fn enters_prepared(_: &mut dyn Game, entering: &mut Entering) -> Result<(), Suspend> {
    entering.prepared = true;
    Ok(())
}
