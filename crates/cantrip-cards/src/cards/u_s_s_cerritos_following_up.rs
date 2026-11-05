use cantrip_api::*;
use cantrip_mechanics::{station, statics, tokens, triggers};
use cantrip_oracle as oracle;

const SIX_PLUS: &str = "6+ | Flying";

/// "When U.S.S. Cerritos enters, create two 1/1 red Officer creature tokens." (CR 111.1, 111.4, 603.6a): the tokens are
/// made whether or not the Cerritos is still there as the trigger resolves.
fn two_officers(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::OFFICER, 2)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(triggers::THIS_ENTERS, two_officers, "When U.S.S. Cerritos enters, create two 1/1 red Officer creature tokens."),
        station::ability("Station (Tap another creature you control: Put charge counters equal to its power on this Spacecraft. Station only as a sorcery. It's an artifact creature at 6+.)"),
    ],
    // The 6+ striation (CR 721.2a-b): at six or more charge counters it is an artifact creature 4/4 with flying.
    statics: &[
        static_ability(Layer::Type, station::charged::<6>, station::is_a_creature, SIX_PLUS),
        static_ability(Layer::Ability, station::charged::<6>, statics::have_flying, SIX_PLUS).continues(),
        static_ability(Layer::PtSet, station::charged::<6>, statics::base_pt::<4, 4>, SIX_PLUS).continues(),
    ],
    ..oracle::U_S_S_CERRITOS_FOLLOWING_UP
};

pub static EXTRA: &[&CardDef] = &[&tokens::OFFICER];
