use cantrip_api::*;
use cantrip_mechanics::{statics, station, triggers::NONCREATURE_SPELL_YOU_CAST};
use cantrip_oracle as oracle;

/// "If this Spacecraft isn't a creature" (CR 603.4): an intervening "if", checked as the ability triggers and again as it
/// resolves, on its own permanent.
fn is_not_a_creature(q: &dyn Query, asker: &Asker) -> bool {
    !q.chars(asker.source).is(CardType::Creature)
}

/// "Put two charge counters on it" (CR 122.6): the ability's source.
fn two_charge_counters(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_counters(item.object, CounterKind::Charge, 2);
    Ok(())
}

/// "Cloaking Device — As long as this Spacecraft isn't a creature, it has hexproof" (CR 702.11b, 613.1f): the ability
/// applies in layer 6, after the types are set, so it reads whether the permanent is a creature already.
fn hexproof_while_not_a_creature(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    if !chars.is(CardType::Creature) {
        chars.keywords = chars.keywords.with(Keyword::Hexproof);
    }
}

/// The 8+ striation, "Flying" with the 4/4 power/toughness box (CR 721.2a-b).
const EIGHT_PLUS: &str = "8+ | Flying (4/4)";

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            NONCREATURE_SPELL_YOU_CAST,
            two_charge_counters,
            "Whenever you cast a noncreature spell, if this Spacecraft isn't a creature, put two charge counters on it.",
        )
        .condition(is_not_a_creature),
        station::ability(
            "Station (Tap another creature you control: Put charge counters equal to its power on this Spacecraft. Station only as a sorcery. It's an artifact creature at 8+.)",
        ),
    ],
    statics: &[
        static_ability(
            Layer::Ability,
            |_, source, affected, _| affected == source,
            hexproof_while_not_a_creature,
            "Cloaking Device \u{2014} As long as this Spacecraft isn't a creature, it has hexproof.",
        ),
        static_ability(Layer::Type, station::charged::<8>, station::is_a_creature, EIGHT_PLUS),
        static_ability(Layer::Ability, station::charged::<8>, statics::have_flying, EIGHT_PLUS).continues(),
        static_ability(Layer::PtSet, station::charged::<8>, statics::base_pt::<4, 4>, EIGHT_PLUS).continues(),
    ],
    ..oracle::ROMULAN_BIRD_OF_PREY
};
