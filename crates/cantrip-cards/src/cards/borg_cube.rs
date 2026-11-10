use cantrip_api::*;
use cantrip_mechanics::{affinity, assimilate, station, statics, targets, triggers};
use cantrip_oracle as oracle;

const STATION_TEXT: &str = "Station (Tap another creature you control: Put charge counters equal to its power on this Spacecraft. Station only as a sorcery. It's an artifact creature at 12+.)";
const TWELVE_PLUS: &str = "12+ | Artifact creatures you control have flying.";

/// "Whenever this Spacecraft enters or attacks, assimilate a creature card exiled with it. If you can't, exile up to one
/// target creature." The trigger's own exile instruction and reference form the linked pair (CR 607.2a); it assimilates a
/// linked creature card if possible, otherwise exiles its legal target and links it for the next trigger. A lone available
/// card is forced; otherwise the controller chooses as the ability resolves (CR 608.2d).
fn assimilate_or_exile(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if assimilate::a_creature_card_exiled_with(g, item)?.is_none() {
        if let Some(creature) = targets::object(g, item, 0) {
            g.exile_with(&[creature], g.linked(item), false);
        }
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    own_cost_changes: &[affinity::for_artifact_creatures],
    abilities: &[
        triggered(triggers::THIS_ENTERS_OR_ATTACKS, assimilate_or_exile, "Whenever this Spacecraft enters or attacks, assimilate a creature card exiled with it. If you can't, exile up to one target creature.")
            .targets(&[targets::CREATURE.optional()]),
        station::ability(STATION_TEXT),
    ],
    statics: &[
        static_ability(Layer::Type, station::charged::<12>, station::is_a_creature, TWELVE_PLUS),
        // Its layer 6 part gives the Cube the striation's static ability, continuing the one effect that began in layer 4 (CR
        // 613.6, 721.2a-b): the Cube has it under an earlier "loses all abilities", and a later one takes it (CR 613.7).
        static_ability(Layer::Ability, station::charged::<12>, station::has::<0>, TWELVE_PLUS).continues(),
        static_ability(Layer::PtSet, station::charged::<12>, statics::base_pt::<6, 6>, TWELVE_PLUS).continues(),
    ],
    // The 12+ striation's static ability (CR 721.2a), which the Cube has while it has twelve or more charge counters.
    delayed: &[granted_static_ability(Layer::Ability, statics::your_artifact_creatures, statics::have_flying, "Artifact creatures you control have flying.")],
    ..oracle::BORG_CUBE
};
