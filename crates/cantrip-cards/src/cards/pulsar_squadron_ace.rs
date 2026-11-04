use cantrip_api::*;
use cantrip_mechanics::effects::plus_one_counter_on_this;
use cantrip_mechanics::library::{self, Rest};
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

/// "Look at the top five cards of your library. You may reveal a Spacecraft card from among them and put it into your hand.
/// Put the rest on the bottom of your library in a random order. If you didn't put a card into your hand this way, put a
/// +1/+1 counter on this creature." (CR 401.4, 701.20): "Spacecraft" is read through this creature's text (CR 612). With
/// nothing taken, whether it was declined, not in the five or the library was empty, the counter goes on.
fn look_for_a_spacecraft(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let spacecraft = g.words(item.object).subtype(Subtype::Spacecraft);
    let taken = library::look_at_top(g, item.controller, item.object, 5, (0, 1), true, |c| c.has_subtype(spacecraft), Rest::BottomRandom)?;
    if taken.is_empty() {
        plus_one_counter_on_this(g, item)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        THIS_ENTERS,
        look_for_a_spacecraft,
        "When this creature enters, look at the top five cards of your library. You may reveal a Spacecraft card from among them and put it into your hand. Put the rest on the bottom of your library in a random order. If you didn't put a card into your hand this way, put a +1/+1 counter on this creature.",
    )],
    ..oracle::PULSAR_SQUADRON_ACE
};
