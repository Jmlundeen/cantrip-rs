use cantrip_api::*;
use cantrip_mechanics::storied;
use cantrip_mechanics::triggers::ward;
use cantrip_oracle as oracle;

/// "As long as you have an enduring story, artifacts and creatures you control have ward {1}" (CR 702.195b, 613.1f): each
/// artifact and creature its controller controls, as the earlier layers made it, while that player has the designation.
fn your_artifacts_and_creatures(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, so_far: &Chars) -> bool {
    let s = q.state();
    storied::you_have_an_enduring_story(s, source)
        && s.obj(affected).controller == s.obj(source).controller
        && (so_far.is(CardType::Artifact) || so_far.is(CardType::Creature))
}

/// Ward {1} (CR 702.21a), the triggered ability this card defines in `delayed`, given in layer 6.
fn grant_ward(view: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    let ward = AbilityRef::delayed(view.cards().current(view.state().obj(source)), 0);
    if !chars.abilities().any(|a| a == ward)
        && let Some(slot) = chars.abilities.iter().position(|a| a.is_none())
    {
        chars.abilities[slot] = Some(ward);
    }
}

pub static CARD: CardDef = CardDef {
    storied: true,
    delayed: &[ward::<1>("Ward {1} (Whenever this permanent becomes the target of a spell or ability an opponent controls, counter it unless that player pays {1}.)")],
    statics: &[static_ability(Layer::Ability, your_artifacts_and_creatures, grant_ward, "As long as you have an enduring story, artifacts and creatures you control have ward {1}.")],
    ..oracle::THORIN_OAKENSHIELD
};
