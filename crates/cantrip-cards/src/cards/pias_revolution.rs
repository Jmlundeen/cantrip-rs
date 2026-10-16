use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

const ASK: Question = Question::new("TakeDamageInsteadOfReturn", "have it deal 3 damage to you?", "If you don't, the artifact card returns to its owner's hand", "Take 3 damage", "Let it return").amount(3).no_default();

/// "Whenever a nontoken artifact is put into your graveyard from the battlefield": "your graveyard" is the owner's
/// (CR 400.3), judged on the board as the artifact last existed (CR 603.10a). The card is the ability's subject.
fn artifact_to_your_graveyard(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { old, new, from, to, .. }
            if from.is(ZoneKind::Battlefield)
                && *to == Zone::graveyard(q.controller(source))
                && !q.obj(*old).token
                && q.chars(*old).is(CardType::Artifact) =>
        {
            Fired::about(Target::Object(*new))
        }
        _ => None,
    }
}

/// The targeted opponent chooses whether this enchantment deals 3 damage to them (CR 608.2d); if not, that card
/// returns to its owner's hand, provided it is still in the graveyard. The choice is theirs either way.
fn return_unless_damaged(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Target::Player(opponent) = item.targets[0] else { return Ok(()) };
    if effects::ask_may(g, opponent, item.object, Purpose::Ask(&ASK))? {
        g.damage(item.object, Target::Player(opponent), 3);
        return Ok(());
    }
    if let Target::Object(card) = item.subject
        && g.obj(card).zone == Zone::graveyard(item.controller)
        && g.obj(card).became.is_none()
    {
        g.return_to_hand(&[card]);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(when(EventMask::ZONE_CHANGE, artifact_to_your_graveyard), return_unless_damaged, "Whenever a nontoken artifact is put into your graveyard from the battlefield, return that card to your hand unless target opponent has this enchantment deal 3 damage to them.")
        .targets(&[targets::OPPONENT])
        .look_back()],
    ..oracle::PIAS_REVOLUTION
};
