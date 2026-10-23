use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "Knockout Blow deals 4 damage to target attacking or blocking creature and you gain 2 life." The only target is checked
/// when the spell resolves (CR 608.2b): if it is no longer legal the spell does not resolve at all and no life is gained.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::damage_target_and_gain_life(g, item, 4, 2)
}

/// "This spell costs {2} less to cast if it targets a red creature" - the cost reduction is shared with Mystical Dispute.
pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::ATTACKING_OR_BLOCKING_CREATURE], effect),
    own_cost_changes: &[targets::less_if_it_targets_color::<2, { Color::Red as u8 }>],
    ..oracle::KNOCKOUT_BLOW
};
