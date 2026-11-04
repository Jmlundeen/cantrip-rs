use cantrip_api::*;
use cantrip_mechanics::{effects, mana, triggers};
use cantrip_oracle as oracle;

const TAP_OR_SACRIFICE: Question = Question::new("TapPermanent", "tap an untapped permanent you control, or sacrifice {subject}?", "", "Tap a permanent", "Sacrifice it");

/// "An untapped permanent you control" as the permanent the payment taps (CR 118.3): any type, and one that came under your
/// control this turn counts (CR 302.6 limits only a creature's {T} symbol). The Bridge itself is one
/// only if something has untapped it since it entered tapped.
fn untapped_permanent_you_control(q: &dyn Query, asker: &Asker, id: ObjectId) -> bool {
    q.controller(id) == asker.controller && !q.obj(id).tapped
}

/// "When this land enters, sacrifice it unless you tap an untapped permanent you control" (CR 118.12a): the payment is made as the
/// ability resolves, and when it is not made the land is sacrificed.
fn sacrifice_unless_you_tap_a_permanent(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if !g.may_pay(item.controller, item.object, item.object, Cost::tap_untapped(1, untapped_permanent_you_control), &TAP_OR_SACRIFICE)? {
        return effects::sacrifice_self(g, item);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enters: Some(mana::enters_tapped),
    abilities: &[
        triggered(
            triggers::THIS_ENTERS,
            sacrifice_unless_you_tap_a_permanent,
            "When this land enters, sacrifice it unless you tap an untapped permanent you control.",
        ),
        mana_ability(TAP, add_any_color(1), "{T}: Add one mana of any color."),
    ],
    ..oracle::COMMAND_BRIDGE
};
