use cantrip_api::*;
use cantrip_mechanics::{station, statics, tokens, triggers};
use cantrip_oracle as oracle;

const SIX_PLUS: &str = "6+ | Flying";

/// "Landfall — Whenever a land you control enters, create a 1/1 red Officer creature token" (CR 603.6a): one token for each
/// land its controller's lands bring in, however they entered (CR 305.1).
fn make_officer(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::OFFICER, 1)?;
    Ok(())
}

/// "Whenever Shuttlecraft Galileo attacks, up to seven target attacking creatures gain flying until end of turn" (CR 601.2c,
/// 611.2a): the targets are chosen as the trigger goes on the stack, and each still attacking and still a creature as it
/// resolves gains flying (CR 608.2b, 506.4).
fn grant_flying(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    cantrip_mechanics::effects::targets_gain(g, item, Keyword::Flying)
}

/// The 6+ striation's layer 6 part (CR 721.2a, 613.6): flying and the attack trigger, together, so that both continue the one
/// effect that began in layer 4 (an earlier "loses all abilities" takes both, CR 613.7).
fn flying_and_the_attack_trigger(view: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    statics::have_flying(view, source, chars);
    station::has::<0>(view, source, chars);
}

/// The 6+ striation holds "Flying" and the attack trigger (CR 721.2a): the card's `delayed` list, given to the Spacecraft while
/// it has six or more charge counters.
pub static CARD: CardDef = CardDef {
    abilities: &[
        station::ability("Station (Tap another creature you control: Put charge counters equal to its power on this Spacecraft. Station only as a sorcery. It's an artifact creature at 6+.)"),
        triggered(triggers::LANDFALL, make_officer, "Landfall — Whenever a land you control enters, create a 1/1 red Officer creature token."),
    ],
    statics: &[
        static_ability(Layer::Type, station::charged::<6>, station::is_a_creature, SIX_PLUS),
        static_ability(Layer::Ability, station::charged::<6>, flying_and_the_attack_trigger, SIX_PLUS).continues(),
        static_ability(Layer::PtSet, station::charged::<6>, statics::base_pt::<3, 3>, SIX_PLUS).continues(),
    ],
    delayed: &[triggered(
        triggers::THIS_ATTACKS,
        grant_flying,
        "Whenever Shuttlecraft Galileo attacks, up to seven target attacking creatures gain flying until end of turn.",
    )
    .targets(&[cantrip_mechanics::targets::ATTACKING_CREATURE.between(0, 7)])],
    ..oracle::SHUTTLECRAFT_GALILEO
};

pub static EXTRA: &[&CardDef] = &[&tokens::OFFICER];
