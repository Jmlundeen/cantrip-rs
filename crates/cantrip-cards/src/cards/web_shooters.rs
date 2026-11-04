use cantrip_api::*;
use cantrip_mechanics::attach::{attached_object, equip};
use cantrip_mechanics::statics::{grant_delayed_ability, plus_pt};
use cantrip_mechanics::triggers::THIS_ATTACKS;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "Equipped creature ... has reach" (CR 702.17, 613.1f) as a static ability's `StaticDef::apply`.
fn have_reach(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.keywords = chars.keywords.with(Keyword::Reach);
}

/// The ability the equipped creature has (CR 613.1f): "Whenever this creature attacks, tap target creature an opponent controls." It is the
/// creature's own, so its controller (not the Equipment's, if they differ) controls it and "an opponent" is that player's opponent
/// (CR 113.8). The target is chosen as it goes on the stack (CR 603.3d) and tapped if still legal as it resolves (CR 608.2b).
pub static CARD: CardDef = CardDef {
    abilities: &[equip(mana("{2}"), "Equip {2}")],
    delayed: &[triggered(THIS_ATTACKS, effects::tap_targets, "Whenever this creature attacks, tap target creature an opponent controls.")
        .targets(&[targets::CREATURE_AN_OPPONENT_CONTROLS])],
    statics: &[
        static_ability(Layer::PtModify, attached_object, plus_pt::<1, 1>, "Equipped creature gets +1/+1."),
        static_ability(Layer::Ability, attached_object, have_reach, "Equipped creature has reach."),
        static_ability(
            Layer::Ability,
            attached_object,
            grant_delayed_ability,
            "Equipped creature has \"Whenever this creature attacks, tap target creature an opponent controls.\"",
        ),
    ],
    ..oracle::WEB_SHOOTERS
};
