use cantrip_api::*;
use cantrip_mechanics::triggers::{THIS_ENTERS, YOUR_END_STEP};
use cantrip_mechanics::{effects, lands, tokens};
use cantrip_oracle as oracle;

/// "Create two 3/3 colorless Robot Villain artifact creature tokens named Doombot."
fn create_two_doombots(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::DOOMBOT, 2)?;
    Ok(())
}

/// "As long as you control an artifact creature or a Plan" (CR 604.1, 613.1f): the source itself, while its controller controls
/// an artifact creature or a permanent with the Plan subtype, each judged as the earlier layers made it.
fn itself_while_you_control_an_artifact_creature_or_a_plan(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, _: &Chars) -> bool {
    source == affected
        && lands::count_you_control(q, source, &|c| (c.is(CardType::Artifact) && c.is(CardType::Creature)) || c.has_subtype(Subtype::Plan)) > 0
}

/// "... Doctor Doom has indestructible" (CR 702.12, layer 6).
fn have_indestructible(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.keywords = chars.keywords.with(Keyword::Indestructible);
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, create_two_doombots, "When Doctor Doom enters, create two 3/3 colorless Robot Villain artifact creature tokens named Doombot."),
        triggered(YOUR_END_STEP, effects::draw_a_card_and_lose_1_life, "At the beginning of your end step, you draw a card and lose 1 life."),
    ],
    statics: &[static_ability(
        Layer::Ability,
        itself_while_you_control_an_artifact_creature_or_a_plan,
        have_indestructible,
        "As long as you control an artifact creature or a Plan, Doctor Doom has indestructible.",
    )],
    ..oracle::DOCTOR_DOOM
};

pub static EXTRA: &[&CardDef] = &[&tokens::DOOMBOT];
