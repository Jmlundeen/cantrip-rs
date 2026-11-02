use cantrip_api::*;
use cantrip_mechanics::{another_untapped_creature_you_control, crew, effects, scry, triggers};
use cantrip_oracle as oracle;

/// "When this Vehicle enters, scry 1." (CR 701.22a).
fn scry_1(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    scry(g, item.controller, item.object, 1)?;
    Ok(())
}

/// "Until end of turn, this Vehicle becomes an artifact creature and gains flying. Put a +1/+1 counter on it." (CR 611.2a, 301.7b,
/// 122.1): the two effects are one instruction, the counter another.
fn becomes_artifact_creature_and_gains_flying(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    instruction(g, |g| {
        g.add_effect(item.object, item.object, EffectKind::AddTypes(TypeSet::of(&[CardType::Artifact, CardType::Creature])), Duration::EndOfTurn);
        g.add_effect(item.object, item.object, EffectKind::AddKeywords(Keywords::of(&[Keyword::Flying])), Duration::EndOfTurn);
    });
    g.end_sentence();
    effects::plus_one_counter_on_this(g, item)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(triggers::THIS_ENTERS, scry_1, "When this Vehicle enters, scry 1."),
        ability(
            tap_untapped(3, another_untapped_creature_you_control),
            becomes_artifact_creature_and_gains_flying,
            "Tap three other untapped creatures you control: Until end of turn, this Vehicle becomes an artifact creature and gains flying. Put a +1/+1 counter on it.",
        ),
        crew::crew(1, "Crew 1"),
    ],
    ..oracle::VOYAGER_GLIDECAR
};
