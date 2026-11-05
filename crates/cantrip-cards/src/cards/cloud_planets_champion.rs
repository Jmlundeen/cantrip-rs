use cantrip_api::*;
use cantrip_mechanics::attach::equip_targeting_this_costs_less;
use cantrip_mechanics::statics;
use cantrip_oracle as oracle;

/// "During your turn, as long as Cloud is equipped" (CR 301.5a, 604.1): the source itself, while its controller is the
/// active player and an Equipment is attached to it, asked again whenever characteristics are computed.
fn itself_equipped_during_your_turn(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, chars: &Chars) -> bool {
    statics::itself_during_your_turn(q, source, affected, chars)
        && q.state().battlefield().iter().any(|&id| q.state().obj(id).attached_to == Some(source) && q.chars(id).has_subtype(Subtype::Equipment))
}

/// "... it has double strike and indestructible" (CR 702.4, 702.12, layer 6).
fn have_double_strike_and_indestructible(q: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    statics::have_double_strike(q, source, chars);
    statics::have_indestructible(q, source, chars);
}

pub static CARD: CardDef = CardDef {
    statics: &[static_ability(
        Layer::Ability,
        itself_equipped_during_your_turn,
        have_double_strike_and_indestructible,
        "During your turn, as long as Cloud is equipped, it has double strike and indestructible.",
    )],
    // "Equip abilities you activate that target Cloud cost {2} less to activate." (CR 601.2f, 602.2b)
    cost_changes: &[equip_targeting_this_costs_less::<2>],
    ..oracle::CLOUD_PLANETS_CHAMPION
};
