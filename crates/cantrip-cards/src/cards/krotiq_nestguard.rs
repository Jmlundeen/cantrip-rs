use cantrip_api::*;
use cantrip_oracle as oracle;

/// "This creature can attack this turn as though it didn't have defender" (CR 702.3b, 609.4).
fn attack_despite_defender(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_effect(item.object, item.object, EffectKind::CanAttackDespiteDefender, Duration::EndOfTurn);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(cost("{2}{G}"), attack_despite_defender, "{2}{G}: This creature can attack this turn as though it didn't have defender.")],
    ..oracle::KROTIQ_NESTGUARD
};
