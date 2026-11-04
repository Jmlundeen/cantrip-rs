use cantrip_api::*;
use cantrip_mechanics::{statics, station, targets, triggers};
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

const NINE_PLUS: &str = "9+ | Flying";

/// "Non-Spacecraft creature": a creature on the battlefield that does not have the Spacecraft subtype (a Spacecraft that
/// is a creature, at 9+ charge counters, is not one).
fn non_spacecraft_creature(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    targets::creature_filter(q, asker, t) && matches!(t, Target::Object(id) if !q.chars(id).has_subtype(Subtype::Spacecraft))
}

/// "Return up to two target non-Spacecraft creatures to their owners' hands": those still legal as it resolves go together
/// (CR 608.2b, 400.7); none chosen or still legal, nothing happens.
fn return_them(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.return_to_hand(&targets::legal_objects(g, item));
    Ok(())
}

/// "Defending player mills four cards" (CR 701.17a, 508.5): the player this Spacecraft attacked, from the attack it made
/// (`Query::defending_player`), also after it has left combat.
fn defending_player_mills(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(player) = g.defending_player(item.object) {
        g.mill(player, 4);
    }
    Ok(())
}

/// The 9+ striation's layer 6 part: flying and the attack trigger, together, so that both continue the one effect that
/// began in layer 4 (CR 613.6, 721.2b) and keep applying under an earlier "loses all abilities".
fn flying_and_the_attack_trigger(q: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    statics::have_flying(q, source, chars);
    station::has::<0>(q, source, chars);
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            THIS_ENTERS,
            return_them,
            "When this Spacecraft enters, return up to two target non-Spacecraft creatures to their owners' hands.",
        )
        .targets(&[up_to(2, non_spacecraft_creature)]),
        station::ability("Station (Tap another creature you control: Put charge counters equal to its power on this Spacecraft. Station only as a sorcery. It's an artifact creature at 9+.)"),
    ],
    statics: &[
        static_ability(Layer::Type, station::charged::<9>, station::is_a_creature, NINE_PLUS),
        static_ability(Layer::Ability, station::charged::<9>, flying_and_the_attack_trigger, NINE_PLUS).continues(),
        static_ability(Layer::PtSet, station::charged::<9>, statics::base_pt::<4, 7>, NINE_PLUS).continues(),
    ],
    // The 9+ striation's attack trigger (CR 721.2a).
    delayed: &[triggered(
        triggers::THIS_ATTACKS,
        defending_player_mills,
        "Whenever this Spacecraft attacks, defending player mills four cards.",
    )],
    ..oracle::SPECIMEN_FREIGHTER
};
