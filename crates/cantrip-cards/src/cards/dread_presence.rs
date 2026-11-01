use cantrip_api::*;
use cantrip_mechanics::{effects, lands, targets, triggers};
use cantrip_oracle as oracle;

/// "This creature deals 2 damage to any target and you gain 2 life." (CR 120.3, 119.3)
fn damage_and_gain(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::damage_target_and_gain_life(g, item, 2, 2)
}

/// "Choose one" (CR 700.2): the mode is chosen as the trigger is put on the stack, and only the second has a target.
static MODAL: ModalDef = modes(&[
    mode(effects::draw_a_card_and_lose_1_life, "You draw a card and you lose 1 life."),
    mode(damage_and_gain, "This creature deals 2 damage to any target and you gain 2 life.").targets(&[targets::ANY]),
]);

pub static CARD: CardDef = CardDef {
    abilities: &[modal_triggered(
        when_asked(EventMask::ZONE_CHANGE, triggers::of_subtype_you_control_enters::<{ lands::sub(Subtype::Swamp) }>),
        &MODAL,
        "Whenever a Swamp you control enters, choose one \u{2014} You draw a card and you lose 1 life; or this creature deals 2 damage to any target and you gain 2 life.",
    )],
    ..oracle::DREAD_PRESENCE
};
