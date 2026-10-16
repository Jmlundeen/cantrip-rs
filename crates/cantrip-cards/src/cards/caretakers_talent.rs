use cantrip_api::*;
use cantrip_mechanics::{class, effects, targets, triggers};
use cantrip_oracle as oracle;

/// "Creature tokens you control", at level 3 (CR 716.2a): a creature as the earlier layers made it, a token, under the
/// Class's controller's control.
fn your_creature_tokens(view: &dyn LayerQuery, source: ObjectId, affected: ObjectId, so_far: &Chars) -> bool {
    let s = view.state();
    s.obj(source).class_level >= 3
        && so_far.is(CardType::Creature)
        && s.obj(affected).token
        && s.obj(affected).controller == s.obj(source).controller
}

fn plus_two_plus_two(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    // CR 613.4c: a modifier only reaches something that has power and toughness to modify.
    chars.power = chars.power.map(|v| v + 2);
    chars.toughness = chars.toughness.map(|v| v + 2);
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(when(EventMask::ZONE_CHANGE, triggers::token_you_control_enters), |g, item| effects::draw(g, item, 1), "Whenever one or more tokens you control enter, draw a card. This ability triggers only once each turn.")
            .triggers_each_turn(1)
            .once_per_batch(),
        class::level_two("{W}", "{W}: Level 2"),
        triggered(class::BECOMES_LEVEL_TWO, cantrip_mechanics::copy::create_token_copy_of_target, "When this Class becomes level 2, create a token that's a copy of target token you control.")
            .targets(&[targets::TOKEN_YOU_CONTROL]),
        class::level_three("{3}{W}", "{3}{W}: Level 3"),
    ],
    statics: &[static_ability(Layer::PtModify, your_creature_tokens, plus_two_plus_two, "Creature tokens you control get +2/+2.")],
    ..oracle::CARETAKERS_TALENT
};
