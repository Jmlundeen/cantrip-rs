use cantrip_api::*;
use cantrip_mechanics::triggers::{self, ward};
use cantrip_mechanics::{effects, lands};
use cantrip_oracle as oracle;

/// "Put three stun counters on it and draw three cards" (CR 122.1d, 121.1): the counters go on this creature only if it is still
/// on the battlefield; the cards are drawn either way.
fn stun_itself_and_draw_three(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_counters(item.object, CounterKind::Stun, 3);
    effects::draw(g, item, 3)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ward::<2>("Ward {2}"),
        triggered(triggers::THIS_ATTACKS, stun_itself_and_draw_three, "Whenever this creature attacks, put three stun counters on it and draw three cards. (If a permanent with a stun counter would become untapped, remove one from it instead.)"),
        triggered(when_asked(EventMask::SPELL_CAST, triggers::subtype_spell_you_cast::<{ lands::sub(Subtype::Turtle) }>), effects::untap_self, "Whenever you cast a Turtle spell, untap this creature."),
    ],
    ..oracle::AMBLING_STORMSHELL
};
