use cantrip_api::*;
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

/// "Whenever an opponent sacrifices an artifact" (CR 701.21a): an artifact that a sacrifice moved from the battlefield under an
/// opponent's control, read as it was when it left, so a Treasure sacrificed as a cost counts. The ability is about that
/// opponent ("them"), who is not a target.
fn an_opponent_sacrifices_an_artifact(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    q.state().opponents(me.controller).find_map(|opponent| {
        let sacrificed = triggers::sacrificed_by(q, opponent, event)?;
        if q.chars(sacrificed).is(CardType::Artifact) { Fired::about(Target::Player(opponent)) } else { None }
    })
}

/// "This creature deals 2 damage to them": to the opponent who sacrificed, dealt by the creature (off its last-known
/// information if it has left the battlefield, CR 608.2h).
fn two_damage_to_them(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.damage(item.object, item.subject, 2);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when_asked(EventMask::ZONE_CHANGE, an_opponent_sacrifices_an_artifact),
        two_damage_to_them,
        "Whenever an opponent sacrifices an artifact, this creature deals 2 damage to them.",
    )
    .on_sacrifice()
    .look_back()],
    ..oracle::VENGEFUL_TRACKER
};
