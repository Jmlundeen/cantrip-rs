use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Target creature has base power and toughness 0/0 until end of turn" (CR 613.4b, 611.2): a layer 7b effect that overrides
/// every other setting of base power and toughness, characteristic-defining abilities included (layer 7a), while counters
/// and pumps still apply on top (layer 7c, 7d). A creature with toughness 0 then dies as a state-based action (CR 704.5f).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        g.add_effect(item.object, creature, EffectKind::SetPt(0, 0), Duration::EndOfTurn);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[targets::CREATURE], effect), ..oracle::MULTIPLY_BY_ZERO };
