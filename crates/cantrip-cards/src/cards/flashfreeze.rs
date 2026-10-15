use cantrip_api::*;
use cantrip_mechanics::counter_target;
use cantrip_oracle as oracle;

/// "target red or green spell": a spell on the stack that is red, green or both, the color words read through this
/// spell's own text (CR 612).
fn red_or_green_spell(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    let words = q.words(asker.source);
    matches!(t, Target::Object(id)
        if q.state().stack.iter().any(|i| i.kind == StackKind::Spell && i.object == id)
            && {
                let colors = q.chars(id).colors;
                colors.has(words.color(Color::Red)) || colors.has(words.color(Color::Green))
            })
}

/// "Counter target red or green spell."
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    counter_target(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[TargetSpec { min: 1, max: 1, filter: red_or_green_spell }], effect }),
    ..oracle::FLASHFREEZE
};
