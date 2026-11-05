use cantrip_api::*;
use cantrip_mechanics::statics::itself;
use cantrip_oracle as oracle;

/// "Xande gets +1/+1 for each noncreature, nonland card in your graveyard" (CR 613.4c, layer 7c): "your" is its controller's, only
/// cards (never tokens, CR 111.7), each judged as the layers make it, and the count taken whenever characteristics are computed.
fn plus_one_per_noncreature_nonland_card_in_your_graveyard(view: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    let state = view.state();
    let n = state
        .zone(Zone::graveyard(state.obj(source).controller))
        .iter()
        .filter(|&&card| {
            let card = view.chars(card);
            !card.is(CardType::Creature) && !card.is(CardType::Land)
        })
        .count() as i32;
    chars.power = chars.power.map(|v| v + n);
    chars.toughness = chars.toughness.map(|v| v + n);
}

pub static CARD: CardDef = CardDef {
    statics: &[static_ability(
        Layer::PtModify,
        itself,
        plus_one_per_noncreature_nonland_card_in_your_graveyard,
        "Xande gets +1/+1 for each noncreature, nonland card in your graveyard.",
    )],
    ..oracle::XANDE_DARK_MAGE
};
