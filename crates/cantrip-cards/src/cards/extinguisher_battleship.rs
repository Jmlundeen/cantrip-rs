use cantrip_api::*;
use cantrip_mechanics::{destroy_target, effects, statics, station, triggers};
use cantrip_oracle as oracle;

/// "Target noncreature permanent" (CR 115.1): a permanent on the battlefield that is not a creature, as the layers have it (a
/// Spacecraft below its station threshold is one).
fn noncreature_permanent(q: &dyn Query, _: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if q.is_on_battlefield(id) && !q.chars(id).is(CardType::Creature))
}

const FIVE_PLUS: &str = "5+ | Flying, trample";

/// "When this Spacecraft enters, destroy target noncreature permanent. Then this Spacecraft deals 4 damage to each creature."
/// (CR 701.8, 120.3, 608.2b-c): the one target is a noncreature permanent, so with it illegal the ability does not resolve and no
/// creature is dealt damage; the damage follows the destruction, as a new sentence, and is dealt to every creature on the
/// battlefield at that moment: a permanent that was a creature only through the destroyed one is no longer one (CR 608.2c).
fn destroy_then_deal_4_damage_to_each_creature(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target(g, item, 0);
    effects::damage_each_creature(g, item, 4);
    Ok(())
}

/// The 5+ striation's keywords (CR 721.2a): "Flying, trample" in the one layer-6 effect.
fn have_flying_and_trample(view: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    statics::have_flying(view, source, chars);
    statics::have_trample(view, source, chars);
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::THIS_ENTERS,
            destroy_then_deal_4_damage_to_each_creature,
            "When this Spacecraft enters, destroy target noncreature permanent. Then this Spacecraft deals 4 damage to each creature.",
        )
        .targets(&[target(noncreature_permanent)]),
        station::ability(
            "Station (Tap another creature you control: Put charge counters equal to its power on this Spacecraft. Station only as a sorcery. It's an artifact creature at 5+.)",
        ),
    ],
    statics: &[
        static_ability(Layer::Type, station::charged::<5>, station::is_a_creature, FIVE_PLUS),
        static_ability(Layer::Ability, statics::itself, have_flying_and_trample, FIVE_PLUS).continues(),
        static_ability(Layer::PtSet, statics::itself, statics::base_pt::<10, 10>, FIVE_PLUS).continues(),
    ],
    ..oracle::EXTINGUISHER_BATTLESHIP
};
