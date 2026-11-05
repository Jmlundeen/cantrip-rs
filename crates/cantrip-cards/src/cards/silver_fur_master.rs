use cantrip_api::*;
use cantrip_mechanics::ninjutsu;
use cantrip_mechanics::statics::{other_creatures_you_control, plus_pt};
use cantrip_oracle as oracle;

/// "Ninjutsu abilities you activate cost {1} less to activate." (CR 601.2f, 602.2b, 118.7a: a reduction is of generic mana
/// only): only the controller's own activations, and only abilities of the class "ninjutsu abilities" (`ActivatedDef::ninjutsu`).
fn ninjutsu_abilities_you_activate_cost_one_less(q: &dyn Query, source: ObjectId, player: PlayerId, costed: Costed, _: &[Target]) -> CostDelta {
    let Costed::Ability(_, ability, _) = costed else { return CostDelta::NONE };
    if player == q.controller(source) && q.activated_ability(ability).is_some_and(|d| d.ninjutsu) {
        CostDelta::less(1)
    } else {
        CostDelta::NONE
    }
}

/// "Other Ninja and Rogue creatures you control" (CR 613.4c, layer 7c): the types are read through the words of this card's
/// text (CR 612.2); the source itself, a Ninja, is left out.
fn other_ninja_and_rogue_creatures_you_control(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, so_far: &Chars) -> bool {
    let words = q.words(source);
    other_creatures_you_control(q, source, affected, so_far)
        && (so_far.has_subtype(words.subtype(Subtype::Ninja)) || so_far.has_subtype(words.subtype(Subtype::Rogue)))
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ninjutsu::ninjutsu(mana("{U}{B}"), "Ninjutsu {U}{B} ({U}{B}, Return an unblocked attacker you control to hand: Put this card onto the battlefield from your hand tapped and attacking.)"),
    ],
    cost_changes: &[ninjutsu_abilities_you_activate_cost_one_less],
    statics: &[static_ability(
        Layer::PtModify,
        other_ninja_and_rogue_creatures_you_control,
        plus_pt::<1, 1>,
        "Other Ninja and Rogue creatures you control get +1/+1.",
    )],
    ..oracle::SILVER_FUR_MASTER
};
