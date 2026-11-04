use cantrip_api::*;
use cantrip_mechanics::{hand, statics, station, triggers::THIS_ATTACKS};
use cantrip_oracle as oracle;

const SEVEN_PLUS: &str = "7+ | Flying";

/// "You may put a creature card from your hand onto the battlefield" (CR 608.2c): nobody is asked with no creature card
/// in hand, and only cards in the controller's own hand count.
fn put_a_creature_card(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    hand::may_put_onto_battlefield(g, item.controller, item.object, |c| c.is(CardType::Creature))?;
    Ok(())
}

/// The 7+ striation's layer 6 part: flying and the attack trigger, together, so that both continue the one effect that
/// began in layer 4 (CR 613.6, 721.2b) and keep applying under an earlier "loses all abilities".
fn flying_and_the_attack_trigger(q: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    statics::have_flying(q, source, chars);
    station::has::<0>(q, source, chars);
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        station::ability("Station (Tap another creature you control: Put charge counters equal to its power on this Spacecraft. Station only as a sorcery. It's an artifact creature at 7+.)"),
    ],
    statics: &[
        static_ability(Layer::Type, station::charged::<7>, station::is_a_creature, SEVEN_PLUS),
        static_ability(Layer::Ability, station::charged::<7>, flying_and_the_attack_trigger, SEVEN_PLUS).continues(),
        static_ability(Layer::PtSet, statics::itself, statics::base_pt::<4, 5>, SEVEN_PLUS).continues(),
    ],
    // The 7+ striation's attack trigger (CR 721.2a).
    delayed: &[triggered(
        THIS_ATTACKS,
        put_a_creature_card,
        "Whenever this Spacecraft attacks, you may put a creature card from your hand onto the battlefield.",
    )],
    ..oracle::SLEDGE_CLASS_SEEDSHIP
};
