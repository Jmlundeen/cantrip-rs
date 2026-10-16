use cantrip_api::*;
use cantrip_mechanics::{choose_color, targets};
use cantrip_oracle as oracle;

const ASK: Question = Question::new("ProtectFromColorless", "protection from colorless?", "If you don't, you choose a color instead", "From colorless", "Choose a color");

fn other_creature_you_control(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    (targets::CREATURE_YOU_CONTROL.filter)(q, asker, t) && t != Target::Object(asker.source)
}

/// Protection from colorless, or from a color chosen as the ability resolves (CR 702.16a, 611.2c): a yes/no for
/// colorless, then on no a color.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    let colorless = g.ask(item.controller, RequestKind::May { source: item.object, purpose: Purpose::Ask(&ASK) })?;
    let kind = if colorless == Answer::Yes {
        EffectKind::ProtectionFromColorless
    } else {
        EffectKind::Protection(ColorSet::of(choose_color(g, item.controller, item.object)?))
    };
    g.add_effect(item.object, creature, kind, Duration::EndOfTurn);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(TAP, effect, "{T}: Another target creature you control gains protection from colorless or from the color of your choice until end of turn.")
        .targets(&[target(other_creature_you_control)])],
    ..oracle::GIVER_OF_RUNES
};
