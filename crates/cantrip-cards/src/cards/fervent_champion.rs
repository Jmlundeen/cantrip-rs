use cantrip_api::*;
use cantrip_mechanics::attach::equip_targeting_this_costs_less;
use cantrip_mechanics::triggers::THIS_ATTACKS;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "Another target attacking Knight you control" (CR 115.1, 506.4): a creature of the Knight type (a word of this
/// card's text, CR 612.2) its controller controls, attacking now, other than this one.
fn another_attacking_knight_you_control(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    let knight = q.words(asker.source).subtype(Subtype::Knight);
    (targets::CREATURE_YOU_CONTROL.filter)(q, asker, t)
        && matches!(t, Target::Object(id) if id != asker.source && q.is_attacking(id) && q.chars(id).has_subtype(knight))
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        THIS_ATTACKS,
        effects::target_creature_gets::<1, 0>,
        "Whenever this creature attacks, another target attacking Knight you control gets +1/+0 until end of turn.",
    )
    .targets(&[target(another_attacking_knight_you_control)])],
    // "Equip abilities you activate that target this creature cost {3} less to activate." (CR 601.2f, 602.2b)
    cost_changes: &[equip_targeting_this_costs_less::<3>],
    ..oracle::FERVENT_CHAMPION
};
