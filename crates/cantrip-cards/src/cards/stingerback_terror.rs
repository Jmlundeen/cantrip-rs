use cantrip_api::*;
use cantrip_mechanics::plot::plot;
use cantrip_mechanics::statics::itself;
use cantrip_oracle as oracle;

/// "This creature gets -1/-1 for each card in your hand" (CR 613.4c, layer 7c): "your" is its controller's, and the count is
/// taken whenever characteristics are computed, so it follows the hand as it changes.
fn minus_one_minus_one_per_card_in_hand(view: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    let state = view.state();
    let n = state.zone(Zone::hand(state.obj(source).controller)).len() as i16;
    chars.power = chars.power.map(|v| v - n);
    chars.toughness = chars.toughness.map(|v| v - n);
}

pub static CARD: CardDef = CardDef {
    statics: &[static_ability(Layer::PtModify, itself, minus_one_minus_one_per_card_in_hand, "This creature gets -1/-1 for each card in your hand.")],
    abilities: &[plot(
        mana("{2}{R}"),
        "Plot {2}{R} (You may pay {2}{R} and exile this card from your hand. Cast it as a sorcery on a later turn without paying its mana cost. Plot only as a sorcery.)",
    )],
    ..oracle::STINGERBACK_TERROR
};
