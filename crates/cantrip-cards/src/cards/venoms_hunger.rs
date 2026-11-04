use cantrip_api::*;
use cantrip_mechanics::{cost_reduction, destroy_target, effects, lands, targets};
use cantrip_oracle as oracle;

/// "Destroy target creature. You gain 2 life." (CR 701.8, 119.3): two sentences, so the life is gained after the destruction is
/// done, whether or not the creature was destroyed (indestructible); a spell whose only target has become illegal is not resolved at
/// all (CR 608.2b), so no life is gained then.
fn destroy_and_gain_2(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target(g, item, 0);
    g.end_sentence();
    effects::gain_life(g, item, 2)
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE], destroy_and_gain_2),
    own_cost_changes: &[cost_reduction::less_if_you_control_subtype::<2, { lands::sub(Subtype::Villain) }>],
    ..oracle::VENOMS_HUNGER
};
