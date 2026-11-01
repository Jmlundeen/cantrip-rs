use cantrip_api::*;
use cantrip_mechanics::library::rest_to_bottom_in_random_order;
use cantrip_mechanics::triggers::THIS_BECOMES_TAPPED;
use cantrip_mechanics::{lands, statics, top_of_library};
use cantrip_oracle as oracle;

const PAY_X: Question = Question::new(
    "RevealForDwarvesAndVehicles",
    "pay {X}?",
    "If you do, reveal the top X cards of your library, put all Dwarf and Vehicle cards from among them into your hand, then put the rest on the bottom of your library in a random order",
    "Pay {cost}",
    "Don't pay",
)
.no_default();

/// "Each Vehicle you control gets +1/+1 as long as it's a creature." (CR 613.4c, 301.7, 612.2): a Vehicle its controller controls
/// that is a creature as the earlier layers made it (crewed, CR 702.122), the type read through the words of this card's text.
/// A Vehicle that is not a creature is not affected, and an opponent's Vehicle is not "yours".
fn your_vehicles_that_are_creatures(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, so_far: &Chars) -> bool {
    so_far.is(CardType::Creature) && statics::your_of_subtype::<{ lands::sub(Subtype::Vehicle) }>(q, source, affected, so_far)
}

/// "Whenever Depala becomes tapped, you may pay {X}. If you do, reveal the top X cards of your library, put all Dwarf and Vehicle
/// cards from among them into your hand, then put the rest on the bottom of your library in a random order." (CR 107.3f, 118.12,
/// 701.20a, 401.4.) X is the payer's to choose as they pay it, 0 included; declining reveals nothing. A library with fewer than X
/// cards is revealed whole. Each card is a Dwarf or Vehicle as the card in the library is, the types read through the words of this
/// card's text; they all go to the hand together, revealed, and the rest go to the bottom.
fn reveal(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let Some(x) = g.may_pay_x(you, item.object, item.object, cost("{X}"), &PAY_X)? else { return Ok(()) };
    let revealed = top_of_library(g, you, usize::from(x));
    if revealed.is_empty() {
        return Ok(());
    }
    g.reveal(you, &revealed);
    let words = g.words(item.object);
    let (dwarf, vehicle) = (words.subtype(Subtype::Dwarf), words.subtype(Subtype::Vehicle));
    let (taken, rest): (Vec<ObjectId>, Vec<ObjectId>) = revealed.into_iter().partition(|&c| {
        let chars = g.chars(c);
        chars.has_subtype(dwarf) || chars.has_subtype(vehicle)
    });
    g.return_to_hand_revealed(&taken);
    rest_to_bottom_in_random_order(g, you, &rest);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_BECOMES_TAPPED, reveal, "Whenever Depala becomes tapped, you may pay {X}. If you do, reveal the top X cards of your library, put all Dwarf and Vehicle cards from among them into your hand, then put the rest on the bottom of your library in a random order.")],
    statics: &[
        static_ability(Layer::PtModify, statics::other_of_subtype_you_control::<{ lands::sub(Subtype::Dwarf) }>, statics::plus_pt::<1, 1>, "Other Dwarves you control get +1/+1."),
        static_ability(Layer::PtModify, your_vehicles_that_are_creatures, statics::plus_pt::<1, 1>, "Each Vehicle you control gets +1/+1 as long as it's a creature."),
    ],
    ..oracle::DEPALA_PILOT_EXEMPLAR
};
