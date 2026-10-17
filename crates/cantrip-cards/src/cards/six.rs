use cantrip_api::*;
use cantrip_mechanics::{library, retrace};
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

/// Mill three cards. You may put a land card from among them into your hand (CR 701.17c); nobody is asked when no
/// land is among them.
fn mill_and_take(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    library::mill_then_may_take_into_hand(g, item.controller, item.object, 3, "land", |c| c.is(CardType::Land))
}

/// During your turn, nonland permanent cards in your graveyard have retrace (CR 702.81a): the permission is Six's, so
/// it reaches the cards in its controller's graveyard whatever they are, and only on that player's own turn.
fn your_nonland_permanent_cards(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, so_far: &Chars) -> bool {
    let controller = q.state().obj(source).controller;
    q.state().turn.active == controller
        && q.state().obj(affected).zone == Zone::graveyard(controller)
        && cantrip_mechanics::targets::is_nonland_permanent_card(so_far)
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(triggers::THIS_ATTACKS, mill_and_take, "Whenever Six attacks, mill three cards. You may put a land card from among them into your hand.")],
    statics: &[static_ability(Layer::Ability, your_nonland_permanent_cards, retrace::give, "During your turn, nonland permanent cards in your graveyard have retrace.")
        .everywhere()],
    ..oracle::SIX
};
