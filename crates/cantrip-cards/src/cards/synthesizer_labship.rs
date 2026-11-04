use cantrip_api::*;
use cantrip_mechanics::statics::have_flying_and_vigilance;
use cantrip_mechanics::triggers::BEGINNING_OF_YOUR_COMBAT;
use cantrip_mechanics::{Animation, animate, statics, station, targets};
use cantrip_oracle as oracle;

const TWO_PLUS: &str = "2+ | At the beginning of combat on your turn, up to one other target artifact you control becomes an artifact creature with base power and toughness 2/2 and gains flying until end of turn.";
const NINE_PLUS: &str = "9+ | Flying, vigilance";

/// "Other target artifact you control": an artifact you control that is not this Spacecraft (CR 115.1, 109.1).
fn other_artifact_you_control(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    (targets::ARTIFACT_YOU_CONTROL.filter)(q, asker, t) && t != Target::Object(asker.source)
}

/// "... becomes an artifact creature with base power and toughness 2/2 and gains flying until end of turn": Artifact and
/// Creature are added (layer 4), flying granted (6) and the base power and toughness set (7b), so counters and pumps count
/// on top and a later setting effect overwrites it. A target no longer legal means the ability does nothing (CR 608.2b).
fn animate_target(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(permanent) = targets::object(g, item, 0) else { return Ok(()) };
    let animation = Animation {
        power: 2,
        toughness: 2,
        colors: None,
        types: TypeSet::of(&[CardType::Artifact]),
        subtypes: &[],
        all_creature_types: false,
        replaces_creature_types: false,
        keywords: Keywords::of(&[Keyword::Flying]),
        ability: None,
    };
    animate(g, item.object, permanent, &animation, Duration::EndOfTurn);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[station::ability(
        "Station (Tap another creature you control: Put charge counters equal to its power on this Spacecraft. Station only as a sorcery. It's an artifact creature at 9+.)",
    )],
    statics: &[
        static_ability(Layer::Type, station::charged::<9>, station::is_a_creature, NINE_PLUS),
        static_ability(Layer::Ability, station::charged::<9>, have_flying_and_vigilance, NINE_PLUS).continues(),
        static_ability(Layer::PtSet, station::charged::<9>, statics::base_pt::<4, 4>, NINE_PLUS).continues(),
        // The 2+ striation's ability (CR 721.2a), the card's first `delayed` ability.
        static_ability(Layer::Ability, station::charged::<2>, station::has::<0>, TWO_PLUS),
    ],
    delayed: &[triggered(BEGINNING_OF_YOUR_COMBAT, animate_target, TWO_PLUS).targets(&[up_to(1, other_artifact_you_control)])],
    ..oracle::SYNTHESIZER_LABSHIP
};
