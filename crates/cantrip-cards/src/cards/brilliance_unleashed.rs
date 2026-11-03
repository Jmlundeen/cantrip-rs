use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "It's a 3/3 Robot artifact creature with flying": what a card that was not an artifact creature enters as (CR 614.12, 205.1b,
/// 613.1d, 613.4b), from the moment it enters, so what watches creatures enter sees it. It keeps its other card types (a land
/// stays a land), loses its creature types and is a Robot.
static AS_A_ROBOT: EntersAs = EntersAs {
    adds_types: TypeSet::of(&[CardType::Artifact, CardType::Creature]),
    loses_subtypes_of: &[CardType::Creature],
    subtypes: &[Subtype::Robot],
    power_toughness: Some((3, 3)),
    keywords: Keywords::of(&[Keyword::Flying]),
    ..EntersAs::NONE
};

/// "Choose target artifact card in your graveyard. Return it to the battlefield if it's an artifact creature card. Otherwise,
/// return it to the battlefield and it's a 3/3 Robot artifact creature with flying." With the target gone or no longer such a
/// card nothing is returned (CR 608.2b).
fn return_the_artifact(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(card) = targets::object(g, item, 0) else { return Ok(()) };
    if g.chars(card).is(CardType::Creature) {
        g.put_onto_battlefield(card, item.controller, false)?;
    } else {
        g.put_onto_battlefield_as(card, item.controller, item, &AS_A_ROBOT)?;
    }
    Ok(())
}

/// "Choose one or both" (CR 700.2): the modes are carried out in the order printed (CR 608.2c).
static MODAL: ModalDef = modes(&[
    mode(effects::damage_first_target::<5>, "Brilliance Unleashed deals 5 damage to target creature.").targets(&[targets::CREATURE]),
    mode(
        return_the_artifact,
        "Choose target artifact card in your graveyard. Return it to the battlefield if it's an artifact creature card. Otherwise, return it to the battlefield and it's a 3/3 Robot artifact creature with flying.",
    )
    .targets(&[targets::ARTIFACT_CARD_IN_YOUR_GRAVEYARD]),
])
.choose(1, 2);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::BRILLIANCE_UNLEASHED };
