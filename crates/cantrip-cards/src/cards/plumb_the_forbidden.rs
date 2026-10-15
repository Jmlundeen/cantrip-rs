use cantrip_api::*;
use cantrip_mechanics::targets::filters;
use cantrip_mechanics::triggers::cast_this_spell;
use cantrip_oracle as oracle;

/// You draw a card and lose 1 life.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.draw(item.controller, 1);
    g.lose_life(item.controller, 1);
    Ok(())
}

/// "When you do" (CR 603.12): the cost of sacrificing creatures was paid, so the spell was "kicked" as many times as
/// creatures were sacrificed for it.
fn sacrificed_creatures(q: &dyn Query, asker: &Asker) -> bool {
    q.obj(asker.source).kicked > 0
}

/// Copy this spell for each creature sacrificed this way (CR 707.10).
fn copy_for_each(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    for _ in 0..g.obj(item.object).kicked {
        g.copy_spell(item.object, item.controller, false)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[], effect }),
    optional_cost: Some(OptionalCost {
        cost: Cost { variable: Some(VariableCost::SacrificeOneOrMore(filters::creature)), ..Cost::NONE },
    }),
    labels: &["Cast without sacrificing", "Sacrifice one or more creatures"],
    abilities: &[AbilityDef::Triggered(TriggeredDef {
        text: "When you do, copy this spell for each creature sacrificed this way.",
        listens: EventMask::SPELL_CAST,
        matches: cast_this_spell,
        condition: Some(sacrificed_creatures),
        effect: copy_for_each,
        anywhere: true,
        ..TriggeredDef::BLANK
    })],
    ..oracle::PLUMB_THE_FORBIDDEN
};
