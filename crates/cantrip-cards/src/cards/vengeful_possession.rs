use cantrip_api::*;
use cantrip_mechanics::effects::may_discard_to_draw;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Gain control of target creature until end of turn (CR 611.2a, 613.2). Untap it. It gains haste until end of turn. You may
/// discard a card. If you do, draw a card." Nothing at all if the target is no longer legal (CR 608.2b), the discard included.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    g.gain_control(item.object, creature, item.controller, Duration::EndOfTurn);
    g.untap(creature);
    g.add_effect(item.object, creature, EffectKind::AddKeywords(Keywords::of(&[Keyword::Haste])), Duration::EndOfTurn);
    may_discard_to_draw(g, item, 1)
}

pub static CARD: CardDef = CardDef { spell: spell(&[targets::CREATURE], effect), ..oracle::VENGEFUL_POSSESSION };
