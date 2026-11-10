use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::triggers::DEALS_COMBAT_DAMAGE_TO_PLAYER;
use cantrip_oracle as oracle;

/// "Geordi La Forge can't be blocked if you control a Spacecraft" (CR 509.1b): whether a block of him is refused is asked of
/// the board as blockers are declared, and only of the permanents his own controller controls - a Spacecraft of the defending
/// player's never counts. A block already made stands (blockers are declared once).
fn cant_block_with_a_spacecraft(q: &dyn Query, source: ObjectId, _: ObjectId, attacker: ObjectId) -> bool {
    !(attacker == source && controls_a_spacecraft(q, q.state().obj(source).controller))
}

fn controls_a_spacecraft(q: &dyn Query, player: PlayerId) -> bool {
    q.state()
        .battlefield()
        .iter()
        .any(|&id| q.state().obj(id).controller == player && q.chars(id).has_subtype(Subtype::Spacecraft))
}

pub static CARD: CardDef = CardDef {
    block_rules: &[cant_block_with_a_spacecraft],
    abilities: &[triggered(
        DEALS_COMBAT_DAMAGE_TO_PLAYER,
        |g, item| effects::draw(g, item, 1),
        "Whenever Geordi La Forge deals combat damage to a player, draw a card.",
    )],
    ..oracle::GEORDI_LA_FORGE_INNOVATOR
};
