use cantrip_api::*;
use cantrip_mechanics::tokens;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

/// "Activated abilities of artifact tokens you control cost {1} less to activate" (CR 601.2f, 118.7): the source of the
/// ability is an artifact token under the controller of this permanent, and the reduction may reach {0}.
fn artifact_token_abilities_cost_less(q: &dyn Query, source: ObjectId, player: PlayerId, costed: Costed, _: &[Target]) -> CostDelta {
    let Costed::Ability(from, ..) = costed else { return CostDelta::NONE };
    if player != q.controller(source) || q.controller(from) != player || !q.obj(from).token {
        return CostDelta::NONE;
    }
    if q.is_on_battlefield(from) && q.chars(from).is(CardType::Artifact) { CostDelta::less(1) } else { CostDelta::NONE }
}

/// When Mutagen Man enters, create X Mutagen tokens, for the X it was cast with (CR 107.3m); entering without being
/// cast, X is 0 and there are none.
fn make_mutagens(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let x = g.obj(item.object).x as u32;
    if x > 0 {
        tokens::create(g, item.controller, &tokens::MUTAGEN, x)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    cost_changes: &[artifact_token_abilities_cost_less],
    abilities: &[triggered(THIS_ENTERS, make_mutagens, "When Mutagen Man enters, create X Mutagen tokens.")],
    ..oracle::MUTAGEN_MAN_LIVING_OOZE
};

pub static EXTRA: &[&CardDef] = &[&tokens::MUTAGEN];
