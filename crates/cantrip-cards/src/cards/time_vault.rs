use cantrip_api::*;
use cantrip_oracle as oracle;

/// "If you would begin your turn while this artifact is tapped" (CR 614.10).
fn tapped_as_its_controllers_turn_would_begin(q: &dyn Query, asker: &Asker, player: PlayerId) -> bool {
    player == asker.controller && q.obj(asker.source).tapped
}

/// "If you do, untap this artifact."
fn untap_it(g: &mut dyn Game, asker: &Asker) -> Result<(), Suspend> {
    g.untap(asker.source);
    Ok(())
}

/// "{T}: Take an extra turn after this one." (CR 500.7.)
fn extra_turn(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_extra_turns(item.controller, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    keywords: Keywords::of(&[Keyword::DoesNotUntap]),
    abilities: &[ability(TAP, extra_turn, "{T}: Take an extra turn after this one.")],
    turn_begin: &[TurnBeginRule { applies: tapped_as_its_controllers_turn_would_begin, on_skip: untap_it }],
    enters: Some(cantrip_mechanics::mana::enters_tapped),
    ..oracle::TIME_VAULT
};
