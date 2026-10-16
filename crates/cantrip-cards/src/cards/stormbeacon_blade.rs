use cantrip_api::*;
use cantrip_mechanics::attach::{attached_object, equip};
use cantrip_oracle as oracle;

/// "Draw a card if you control three or more attacking creatures": the ability always triggers (it is no intervening
/// "if", CR 603.4); the count is taken as it resolves.
fn draw_if_three(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let attacking = g
        .permanents_of(item.controller)
        .into_iter()
        .filter(|&id| g.chars(id).is(CardType::Creature) && g.is_attacking(id))
        .count();
    if attacking >= 3 {
        g.draw(item.controller, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(cantrip_mechanics::triggers::EQUIPPED_CREATURE_ATTACKS, draw_if_three, "Whenever equipped creature attacks, draw a card if you control three or more attacking creatures."),
        equip(mana("{2}"), "Equip {2}"),
    ],
    statics: &[static_ability(Layer::PtModify, attached_object, cantrip_mechanics::statics::plus_pt::<3, 0>, "Equipped creature gets +3/+0.")],
    ..oracle::STORMBEACON_BLADE
};
