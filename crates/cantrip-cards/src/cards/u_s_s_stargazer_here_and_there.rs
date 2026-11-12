use cantrip_api::*;
use cantrip_mechanics::combat::create_attacking_of_your_choice;
use cantrip_mechanics::triggers::THIS_ATTACKS;
use cantrip_mechanics::tokens::SPACECRAFT_ILLUSION;
use cantrip_mechanics::{delayed, station, statics};
use cantrip_oracle as oracle;

const SIX_PLUS: &str = "6+ | Flying";

/// "Picard Maneuver — Whenever U.S.S. Stargazer attacks, create a tapped and attacking 4/3 colorless Spacecraft Illusion
/// artifact creature token with flying. Exile that token at end of combat." (CR 508.4, 603.6a, 603.7): the token is never
/// declared as an attacker, and the delayed ability at end of combat is about the token made (CR 400.7).
fn picard_maneuver(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let token = g.card_id(&SPACECRAFT_ILLUSION);
    let made = create_attacking_of_your_choice(g, item.controller, item.object, token, 1, true)?;
    if made.is_empty() { return Ok(()); }
    g.delayed_trigger_about(
        DelayedWhen::Step { step: Step::EndOfCombat, on_turn_of: None },
        item.object,
        item.controller,
        AbilityRef::delayed(item.ability.card, 0),
        &made,
    );
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        station::ability("Station (Tap another creature you control: Put charge counters equal to its power on this Spacecraft. Station only as a sorcery. It's an artifact creature at 6+.)"),
        triggered(
            THIS_ATTACKS,
            picard_maneuver,
            "Picard Maneuver \u{2014} Whenever U.S.S. Stargazer attacks, create a tapped and attacking 4/3 colorless Spacecraft Illusion artifact creature token with flying. Exile that token at end of combat.",
        ),
    ],
    statics: &[
        // "{6+}[abilities][P/T]" (CR 721.2a-b): at six or more charge counters it is an artifact creature with base power and
        // toughness 4/3 in addition to its other types, with flying.
        static_ability(Layer::Type, station::charged::<6>, station::is_a_creature, SIX_PLUS),
        static_ability(Layer::Ability, station::charged::<6>, statics::have_flying, SIX_PLUS).continues(),
        static_ability(Layer::PtSet, station::charged::<6>, statics::base_pt::<4, 3>, SIX_PLUS).continues(),
    ],
    delayed: &[delayed_trigger(delayed::exile_subject, "Exile that token at end of combat.")],
    ..oracle::U_S_S_STARGAZER_HERE_AND_THERE
};

pub static EXTRA: &[&CardDef] = &[&SPACECRAFT_ILLUSION];
