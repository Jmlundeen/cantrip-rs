use cantrip_api::*;
use cantrip_mechanics::attach::{attached_object, equip};
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "This ability costs {X} less to activate, where X is this creature's power." (CR 601.2f by way of 602.2b): the power of the
/// creature that has the ability, as the cost is determined - before any of it is paid; a power below zero (CR 107.1b) reduces
/// nothing.
fn less_by_power(q: &dyn Query, asker: &Asker, _: Costed, _: &[Target]) -> CostDelta {
    CostDelta::less(q.chars(asker.source).power.unwrap_or(0).max(0) as u32)
}

pub static CARD: CardDef = CardDef {
    abilities: &[equip(mana("{1}"), "Equip {1}")],
    // "Exile The Dominion Bracelet" in the ability a Bracelet gives (CR 201.5a): the Bracelet that gave this instance,
    // whoever controls it (only a sacrifice needs the payer's permanent, CR 701.21a).
    delayed: &[ability(
        cost("{15}").and(Cost { exile_granter: true, ..Cost::NONE }),
        effects::control_target_player_next_turn,
        "{15}, Exile The Dominion Bracelet: You control target opponent during their next turn. This ability costs {X} less to activate, where X is this creature's power. Activate only as a sorcery.",
    )
    .targets(&[targets::OPPONENT])
    .sorcery_speed()
    .cost_change(less_by_power)],
    statics: &[
        static_ability(Layer::PtModify, attached_object, cantrip_mechanics::statics::plus_pt::<1, 1>, "Equipped creature gets +1/+1."),
        static_ability(
            Layer::Ability,
            attached_object,
            cantrip_mechanics::statics::grant_delayed_ability,
            "Equipped creature has \"{15}, Exile The Dominion Bracelet: You control target opponent during their next turn. This ability costs {X} less to activate, where X is this creature's power. Activate only as a sorcery.\"",
        ),
    ],
    ..oracle::THE_DOMINION_BRACELET
};
