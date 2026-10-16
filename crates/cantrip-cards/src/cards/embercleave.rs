use cantrip_api::*;
use cantrip_mechanics::{attach::{attach_to_target, attached_object, equip}, targets, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// This spell costs {1} less to cast for each attacking creature you control (CR 601.2f, 506.4): generic mana only,
/// and nothing outside combat.
fn cheaper_per_attacker(q: &dyn Query, _: ObjectId, player: PlayerId, costed: Costed) -> CostDelta {
    if !matches!(costed, Costed::Spell(_)) {
        return CostDelta::NONE;
    }
    let attackers = q
        .state()
        .attackers
        .iter()
        .filter(|a| q.is_on_battlefield(a.id) && q.controller(a.id) == player && q.chars(a.id).is(CardType::Creature))
        .count();
    CostDelta::less(attackers.min(255) as u8)
}

fn double_strike_and_trample(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.keywords = chars.keywords.with(Keyword::DoubleStrike).with(Keyword::Trample);
}

pub static CARD: CardDef = CardDef {
    own_cost_changes: &[cheaper_per_attacker],
    abilities: &[
        triggered(THIS_ENTERS, attach_to_target, "When Embercleave enters, attach it to target creature you control.")
            .targets(&[targets::CREATURE_YOU_CONTROL]),
        equip(mana("{3}"), "Equip {3}"),
    ],
    statics: &[
        static_ability(Layer::PtModify, attached_object, cantrip_mechanics::statics::plus_pt::<1, 1>, "Equipped creature gets +1/+1."),
        static_ability(Layer::Ability, attached_object, double_strike_and_trample, "Equipped creature has double strike and trample."),
    ],
    ..oracle::EMBERCLEAVE
};
