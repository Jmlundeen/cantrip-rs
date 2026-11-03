use cantrip_api::*;
use cantrip_mechanics::{protection, statics};
use cantrip_oracle as oracle;

/// Creatures named Ivory Guardians while an opponent controls a nontoken red permanent (CR 604.1, 612.2).
fn guardians_with_opponent_red(q: &dyn LayerQuery, source: ObjectId, _: ObjectId, chars: &Chars) -> bool {
    chars.is(CardType::Creature)
        && chars.name == "Ivory Guardians"
        && q.state().opponents(q.controller(source)).any(|opponent| {
            q.state().battlefield().iter().copied().any(|id| {
                q.controller(id) == opponent
                    && !q.state().obj(id).token
                    && q.chars(id).colors.has(q.words(source).color(Color::Red))
            })
        })
}

pub static CARD: CardDef = CardDef {
    statics: &[
        protection::from::<{ Color::Red as u8 }>("Protection from red"),
        static_ability(
            Layer::PtModify,
            guardians_with_opponent_red,
            statics::plus_pt::<1, 1>,
            "Creatures named Ivory Guardians get +1/+1 as long as an opponent controls a nontoken red permanent.",
        ),
    ],
    ..oracle::IVORY_GUARDIANS
};
