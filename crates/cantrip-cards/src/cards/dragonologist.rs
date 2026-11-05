use cantrip_api::*;
use cantrip_mechanics::library::{look_at_top, Rest};
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{lands, statics};
use cantrip_oracle as oracle;

/// "When this creature enters, look at the top six cards of your library. You may reveal an instant, sorcery, or Dragon card from
/// among them and put it into your hand. Put the rest on the bottom of your library in a random order." (CR 401.4, 701.20a) The
/// creature type is read through the words of this card's text (CR 612.2).
fn look(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let dragon = g.words(item.object).subtype(Subtype::Dragon);
    let wanted = |c: &Chars| c.is(CardType::Instant) || c.is(CardType::Sorcery) || c.has_subtype(dragon);
    look_at_top(g, item.controller, item.object, 6, (0, 1), true, wanted, Rest::BottomRandom)?;
    Ok(())
}

/// "Untapped Dragons you control" (CR 613.1f, 110.5): a Dragon its controller controls that is untapped now.
fn your_untapped_dragons(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, so_far: &Chars) -> bool {
    statics::your_of_subtype::<{ lands::sub(Subtype::Dragon) }>(q, source, affected, so_far) && !q.state().obj(affected).tapped
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        THIS_ENTERS,
        look,
        "When this creature enters, look at the top six cards of your library. You may reveal an instant, sorcery, or Dragon card from among them and put it into your hand. Put the rest on the bottom of your library in a random order.",
    )],
    statics: &[static_ability(Layer::Ability, your_untapped_dragons, statics::have_hexproof, "Untapped Dragons you control have hexproof.")],
    ..oracle::DRAGONOLOGIST
};
