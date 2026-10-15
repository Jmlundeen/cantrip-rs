use cantrip_api::*;
use cantrip_mechanics::{targets, tokens};
use cantrip_oracle as oracle;

/// "Create a Gold token. Each opponent attacking that player does the same." Who is attacking that player is read as the
/// ability resolves (CR 506.2): each opponent of the Aura's controller with a creature still attacking them.
fn gold(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(enchanted) = g.obj(item.object).attached_player else { return Ok(()) };
    let token = g.card_id(&tokens::GOLD);
    let mut makers = vec![item.controller];
    let opponents: Vec<PlayerId> = g.state().opponents(item.controller).collect();
    let attackers: Vec<(ObjectId, Target)> = g.state().attackers.iter().map(|a| (a.id, a.target)).collect();
    for p in opponents {
        if attackers.iter().any(|&(id, target)| target == Target::Player(enchanted) && g.controller(id) == p) {
            makers.push(p);
        }
    }
    let tokens: Vec<(CardId, PlayerId)> = makers.into_iter().map(|p| (token, p)).collect();
    g.create_tokens_together(&tokens)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[AbilityDef::Triggered(TriggeredDef {
        text: "Whenever enchanted player is attacked, create a Gold token. Each opponent attacking that player does the same.",
        listens: EventMask::ATTACKS,
        matches: cantrip_mechanics::triggers::enchanted_player_attacked,
        effect: gold,
        once_per_batch: true,
        ..TriggeredDef::BLANK
    })],
    enchant: Some(targets::PLAYER),
    ..oracle::CURSE_OF_OPULENCE
};

pub static EXTRA: &[&CardDef] = &[&tokens::GOLD];
