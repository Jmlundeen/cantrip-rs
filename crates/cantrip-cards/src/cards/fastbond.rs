use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Whenever you play a land" (CR 305.1): a land of yours put onto the battlefield by the special action, not by an
/// effect.
fn you_play_a_land(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { new, to, reason: ZoneReason::PlayLand, .. }
            if to.is(ZoneKind::Battlefield) && q.controller(*new) == q.controller(source) =>
        {
            Fired::PLAIN
        }
        _ => None,
    }
}

/// "If it wasn't the first land you played this turn" (CR 603.4): the turn's count already includes this land.
fn not_the_first(q: &dyn Query, asker: &Asker) -> bool {
    q.state().player(asker.controller).lands_played > 1
}

fn burn(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.damage(item.object, Target::Player(item.controller), 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[AbilityDef::Triggered(TriggeredDef {
        text: "Whenever you play a land, if it wasn't the first land you played this turn, this enchantment deals 1 damage to you.",
        listens: EventMask::ZONE_CHANGE,
        matches: you_play_a_land,
        effect: burn,
        condition: Some(not_the_first),
        ..TriggeredDef::BLANK
    })],
    play_rules: &[PlayRule::AnyNumberOfLandPlays],
    ..oracle::FASTBOND
};
