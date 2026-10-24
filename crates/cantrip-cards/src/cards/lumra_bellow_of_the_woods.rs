use cantrip_api::*;
use cantrip_mechanics::lands::{self, count_you_control};
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

/// "Lumra's power and toughness are each equal to the number of lands you control" (CR 604.3, layer 7a): a
/// characteristic-defining ability, so it works in every zone, counting its owner's lands off the battlefield (the ruling).
fn size_is_lands_you_control(q: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    let lands = count_you_control(q, source, &|c| c.is(CardType::Land));
    chars.power = Some(lands);
    chars.toughness = Some(lands);
}

/// "When Lumra enters, mill four cards. Then return all land cards from your graveyard to the battlefield tapped": the
/// lands (including those just milled) are returned after the milling, which ends first (CR 608.2c).
fn mill_then_return_lands(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    g.mill(you, 4);
    g.end_sentence();
    lands::return_all_land_cards_from_graveyard_tapped(g, you)
}

pub static CARD: CardDef = CardDef {
    statics: &[characteristic_defining(
        Layer::PtDefine,
        size_is_lands_you_control,
        "Lumra's power and toughness are each equal to the number of lands you control.",
    )
    .lost_pt(0, 0)],
    abilities: &[triggered(
        THIS_ENTERS,
        mill_then_return_lands,
        "When Lumra enters, mill four cards. Then return all land cards from your graveyard to the battlefield tapped.",
    )],
    ..oracle::LUMRA_BELLOW_OF_THE_WOODS
};
