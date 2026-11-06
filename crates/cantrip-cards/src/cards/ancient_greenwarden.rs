use cantrip_api::*;
use cantrip_mechanics::statics;
use cantrip_oracle as oracle;

/// "You may play lands from your graveyard. If a land entering causes a triggered ability of a permanent you control to trigger, that ability triggers an additional time." (CR 604.1).
fn play_lands_graveyard_if_land_trigger_rule(
    q: &dyn Query,
    asker: &Asker,
    event: &Event,
    triggering: &Triggering,
) -> bool {
    (triggering.permanent && asker.controller == triggering.controller)
        && statics::entering(event).is_some_and(|id| q.chars(id).is(CardType::Land))
}

pub static CARD: CardDef = CardDef {
    additional_triggers: &[play_lands_graveyard_if_land_trigger_rule],
    play_rules: &[PlayRule::LandsFromGraveyard],
    ..oracle::ANCIENT_GREENWARDEN
};
