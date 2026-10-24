use cantrip_api::*;
use cantrip_mechanics::statics::itself;
use cantrip_mechanics::{library, select, targets};
use cantrip_oracle as oracle;

/// "This creature gets +1/+1 for each creature card in your graveyard" (CR 613.4c, layer 7c): "your" is its controller's, and
/// the count is taken whenever characteristics are computed. Tokens are never in a graveyard as cards (CR 111.7).
fn plus_one_per_creature_card(view: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    let state = view.state();
    let n = state
        .zone(Zone::graveyard(state.obj(source).controller))
        .iter()
        .filter(|&&id| view.chars(id).is(CardType::Creature))
        .count() as i16;
    chars.power = chars.power.map(|v| v + n);
    chars.toughness = chars.toughness.map(|v| v + n);
}

/// "Search your library for a creature card with mana value X or less, put it onto the battlefield, then shuffle." X is the value
/// chosen as the ability was activated (CR 107.3); a card in a library with {X} in its cost has X = 0 (CR 107.3g).
fn search(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let rule = select::creature_cards(select::up_to(1)).with(select::mana_value_at_most(item.x as u32));
    library::find_to_battlefield(g, item.controller, item.object, rule, false)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    statics: &[static_ability(Layer::PtModify, itself, plus_one_per_creature_card, "This creature gets +1/+1 for each creature card in your graveyard.")],
    abilities: &[ability(cost("{X}{B/G}, {T}").and(sacrifice(1, targets::filters::another_creature)), search, "{X}{B/G}, {T}, Sacrifice another creature: Search your library for a creature card with mana value X or less, put it onto the battlefield, then shuffle. Activate only as a sorcery.")
        .sorcery_speed()],
    ..oracle::FIEND_ARTISAN
};
