use cantrip_api::*;
use cantrip_mechanics::lands::destroy_target_its_controller_may_fetch_basic;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "Target artifact or land an opponent controls" (CR 115.1): a permanent still the object it was (CR 400.7) that is an
/// artifact or a land, and whose controller is not the caster. Judged again as the spell resolves (CR 608.2b).
fn artifact_or_land_an_opponent_controls(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if targets::permanent(q, id) && q.controller(id) != asker.controller && {
        let c = q.chars(id);
        c.is(CardType::Artifact) || c.is(CardType::Land)
    })
}

/// "Destroy target artifact or land an opponent controls. Its controller may search their library for a basic land card,
/// put it onto the battlefield tapped, then shuffle. Draw a card." The spell's only target being illegal, none of it
/// happens, the draw included (CR 608.2b); otherwise the permanent's last controller (CR 608.2h) is asked after the
/// destruction, even if it survived (indestructible, regenerated), and the draw is the spell's controller's.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target_its_controller_may_fetch_basic(g, item, 0, true)?;
    effects::draw(g, item, 1)
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[target(artifact_or_land_an_opponent_controls)], effect),
    ..oracle::PRICE_OF_FREEDOM
};
