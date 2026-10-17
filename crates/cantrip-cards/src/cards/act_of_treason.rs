use cantrip_api::*;
use cantrip_mechanics::{effects::gain_control_untap_haste, targets};
use cantrip_oracle as oracle;

/// "Gain control of target creature until end of turn. Untap that creature. It gains haste until end of turn." (CR 613.1b,
/// 611.2a, 702.10): any creature, one its caster already controls too (the card's ruling); nothing if the target is no
/// longer legal (CR 608.2b). Its Auras and Equipment do not come with it (the ruling).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    gain_control_untap_haste(g, item);
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[targets::CREATURE], effect), ..oracle::ACT_OF_TREASON };
