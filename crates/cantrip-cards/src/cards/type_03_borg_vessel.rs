use cantrip_api::*;
use cantrip_mechanics::{station, statics, tokens, triggers};
use cantrip_oracle as oracle;

const SEVEN_PLUS: &str = "7+ | Flying";

/// "Artifact spells you cast cost {1} less to cast" (CR 601.2f): only its controller's artifact spells, judged on the
/// spell's own characteristics as it is cast; the reduction takes only the generic part (CR 118.7c).
fn artifact_spells_cost_one_less(q: &dyn Query, source: ObjectId, player: PlayerId, costed: Costed, _: &[Target]) -> CostDelta {
    cantrip_mechanics::cost_reduction::spells_you_cast(q, source, player, costed, CardType::Artifact, 1)
}

/// "Whenever you cast a spell, create a 1/1 colorless Borg artifact creature token" (CR 603.2, 111.10b): one token for
/// each spell its controller casts, this Spacecraft's own cast not among them (it is not on the battlefield yet).
fn create_borg(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::BORG, 1)?;
    Ok(())
}

/// The 7+ striation's layer 6 part (CR 721.2a, 613.6): flying and the cast trigger, together, so that both continue the one
/// effect that began in layer 4 (an earlier "loses all abilities" takes both, CR 613.7).
fn flying_and_the_cast_trigger(view: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    statics::have_flying(view, source, chars);
    station::has::<0>(view, source, chars);
}

/// The 7+ striation holds "Flying" and the triggered ability (CR 721.2a): the card's `delayed` list, given to the Spacecraft
/// while it has seven or more charge counters.
pub static CARD: CardDef = CardDef {
    cost_changes: &[artifact_spells_cost_one_less],
    abilities: &[
        station::ability("Station (Tap another creature you control: Put charge counters equal to its power on this Spacecraft. Station only as a sorcery. It's an artifact creature at 7+.)"),
    ],
    statics: &[
        static_ability(Layer::Type, station::charged::<7>, station::is_a_creature, SEVEN_PLUS),
        static_ability(Layer::Ability, station::charged::<7>, flying_and_the_cast_trigger, SEVEN_PLUS).continues(),
        static_ability(Layer::PtSet, station::charged::<7>, statics::base_pt::<5, 5>, SEVEN_PLUS).continues(),
    ],
    delayed: &[triggered(triggers::YOU_CAST_A_SPELL, create_borg, "Whenever you cast a spell, create a 1/1 colorless Borg artifact creature token.")],
    ..oracle::TYPE_03_BORG_VESSEL
};

pub static EXTRA: &[&CardDef] = &[&tokens::BORG];
