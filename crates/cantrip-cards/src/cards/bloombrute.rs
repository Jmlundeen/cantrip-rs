use cantrip_api::*;
use cantrip_mechanics::{effects, targets, triggers};
use cantrip_oracle as oracle;

/// "Whenever you gain life, draw a card."
fn draw_one(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::draw(g, item, 1)
}

/// "{4}{G}{W}: Target creature gains trample and lifelink until end of turn." (CR 611.2a, 702.15, 702.19): both keywords
/// from one layer 6 effect for the rest of the turn.
fn grant_keywords(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        g.add_effect(item.object, creature, EffectKind::AddKeywords(Keywords::of(&[Keyword::Trample, Keyword::Lifelink])), Duration::EndOfTurn);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(triggers::YOU_GAIN_LIFE, draw_one, "Whenever you gain life, draw a card. This ability triggers only once each turn.")
            .triggers_each_turn(1),
        ability(cost("{4}{G}{W}"), grant_keywords, "{4}{G}{W}: Target creature gains trample and lifelink until end of turn.")
            .targets(&[targets::CREATURE]),
    ],
    ..oracle::BLOOMBRUTE
};
