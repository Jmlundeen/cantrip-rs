use cantrip_api::*;
use cantrip_mechanics::statics::{plus_pt, your_of_subtype};
use cantrip_mechanics::{effects, lands, triggers};
use cantrip_oracle as oracle;

/// "If you attacked with a Hero this turn or a Hero entered the battlefield under your control this turn" (CR 603.4, an
/// intervening "if", checked as the ability triggers and again as it resolves): a Hero declared as an attacker by this
/// ability's controller as it was declared (CR 508.1), or one that entered under their control as it entered (CR 603.6a,
/// 614.12), whatever it has become, wherever it has gone and whoever controls it since. "Hero" is read through the words of
/// this card's text (CR 612).
fn hero_attacked_or_entered(q: &dyn Query, asker: &Asker) -> bool {
    let hero = q.words(asker.source).subtype(Subtype::Hero);
    let history = q.history();
    history.attacked_with(asker.controller).any(|a| a.chars(q).has_subtype(hero))
        || history.entered().any(|e| e.controller == asker.controller && e.chars(q).has_subtype(hero))
}

/// "Draw a card."
fn draw_a_card(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::draw(g, item, 1)
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::EACH_END_STEP,
        draw_a_card,
        "At the beginning of each end step, if you attacked with a Hero this turn or a Hero entered the battlefield under your control this turn, draw a card.",
    )
    .condition(hero_attacked_or_entered)],
    statics: &[static_ability(Layer::PtModify, your_of_subtype::<{ lands::sub(Subtype::Hero) }>, plus_pt::<2, 2>, "Heroes you control get +2/+2.")],
    ..oracle::AVENGERS_ASSEMBLE
};
