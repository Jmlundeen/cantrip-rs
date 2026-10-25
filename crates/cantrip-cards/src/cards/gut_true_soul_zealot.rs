use cantrip_api::*;
use cantrip_mechanics::choose;
use cantrip_mechanics::combat::choose_attack_target;
use cantrip_oracle as oracle;

use super::undercity::SKELETON;

/// "You may sacrifice another creature or an artifact. If you do, create a 4/1 black Skeleton creature token with menace
/// that's tapped and attacking." "Another" belongs to the creature alone: Gut that has become an artifact may be
/// sacrificed to its own ability. Declining is the empty answer to an up-to-one choice, and nobody is asked when nothing
/// is eligible. The token attacks what its controller chooses as it enters (CR 508.4).
fn skeleton(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let gut = item.object;
    let eligible: Vec<ObjectId> = g
        .state()
        .battlefield()
        .iter()
        .copied()
        .filter(|&id| {
            g.controller(id) == you && {
                let c = g.chars(id);
                c.is(CardType::Artifact) || (c.is(CardType::Creature) && id != gut)
            }
        })
        .collect();
    let chosen = choose::cards(g, you, gut, Purpose::Sacrifice, eligible, 1, true)?;
    let Some(&victim) = chosen.first() else { return Ok(()) };
    if g.sacrifice(you, &[victim]).is_empty() {
        return Ok(());
    }
    let token = g.card_id(&SKELETON);
    match choose_attack_target(g, you, gut)? {
        Some(defender) => {
            g.create_tokens_attacking(&[(token, you)], true, defender)?;
        }
        None => {
            g.create_tokens(token, you, 1, true)?;
        }
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(cantrip_mechanics::triggers::YOU_ATTACK, skeleton, "Whenever you attack, you may sacrifice another creature or an artifact. If you do, create a 4/1 black Skeleton creature token with menace that's tapped and attacking.")],
    ..oracle::GUT_TRUE_SOUL_ZEALOT
};
