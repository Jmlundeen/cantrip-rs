use cantrip_api::*;
use cantrip_mechanics::attach::{attached_object, equip};
use cantrip_mechanics::{effects, protection, targets, triggers};
use cantrip_oracle as oracle;

/// "Equipped creature gets +2/+2 and has protection from white and from black." (CR 702.16, layer 6): this static grants
/// only the protection part; the +2/+2 lives on its own PtModify static.
fn protection_from_white_and_black(view: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    protection::from_color::<{ Color::White as u8 }>(view, source, chars);
    protection::from_color::<{ Color::Black as u8 }>(view, source, chars);
}

/// "Whenever equipped creature deals combat damage to a player, you gain 3 life and you may return up to one target
/// creature card from your graveyard to your hand." (CR 608.2c, 603.5): the life first, then the card if the controller
/// chooses to; a declined or absent target still gains the life.
fn gain_then_return(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.gain_life(item.controller, 3);
    g.end_sentence();
    effects::may_return_first_target_to_hand(g, item)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::EQUIPPED_CREATURE_DEALS_COMBAT_DAMAGE_TO_PLAYER,
            gain_then_return,
            "Whenever equipped creature deals combat damage to a player, you gain 3 life and you may return up to one target creature card from your graveyard to your hand.",
        )
        .targets(&[targets::CREATURE_CARD_IN_YOUR_GRAVEYARD.between(0, 1)]),
        equip(mana("{2}"), "Equip {2}"),
    ],
    statics: &[
        static_ability(Layer::PtModify, attached_object, cantrip_mechanics::statics::plus_pt::<2, 2>, "Equipped creature gets +2/+2."),
        static_ability(Layer::Ability, attached_object, protection_from_white_and_black, "has protection from white and from black."),
    ],
    ..oracle::SWORD_OF_LIGHT_AND_SHADOW
};
