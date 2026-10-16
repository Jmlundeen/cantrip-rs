use cantrip_api::*;
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

/// "This creature attacks each combat if able" (CR 508.1d): a static ability of the creature itself.
fn is_self(_: &dyn LayerQuery, source: ObjectId, affected: ObjectId, _: &Chars) -> bool {
    source == affected
}

fn must_attack(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.must_attack = true;
}

/// Discard your hand (all of it, no choice), then draw three cards.
fn rummage(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.discard_hand(item.controller);
    g.draw(item.controller, 3);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(triggers::THIS_ATTACKS, rummage, "Whenever this creature attacks, discard your hand, then draw three cards.")],
    statics: &[static_ability(Layer::Ability, is_self, must_attack, "This creature attacks each combat if able.")],
    madness: Some(mana("{1}{R}")),
    ..oracle::ANJES_RAVAGER
};
