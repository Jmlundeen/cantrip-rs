use cantrip_api::*;
use cantrip_mechanics::effects::may_return_this_card_to_battlefield;
use cantrip_mechanics::statics::{have_haste, itself};
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

/// "As long as an opponent has 10 or less life": any one of its controller's opponents (CR 102.2), counted as the layers
/// are applied.
fn an_opponent_has_ten_or_less_life(view: &dyn LayerQuery, source: ObjectId, affected: ObjectId, so_far: &Chars) -> bool {
    let state = view.state();
    itself(view, source, affected, so_far) && state.opponents(state.obj(source).controller).any(|p| state.player(p).life <= 10)
}

pub static CARD: CardDef = CardDef {
    keywords: Keywords::of(&[Keyword::CantBlock]),
    abilities: &[triggered(
        when_asked(EventMask::ZONE_CHANGE, triggers::landfall),
        may_return_this_card_to_battlefield,
        "Landfall \u{2014} Whenever a land you control enters, you may return this card from your graveyard to the battlefield.",
    )
    // It returns this card from the graveyard, so it functions only there (CR 113.6m).
    .functions_in(IN_A_GRAVEYARD)],
    statics: &[static_ability(
        Layer::Ability,
        an_opponent_has_ten_or_less_life,
        have_haste,
        "This creature has haste as long as an opponent has 10 or less life.",
    )],
    ..oracle::BLOODGHAST
};
