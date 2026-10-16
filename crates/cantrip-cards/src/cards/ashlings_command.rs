use cantrip_api::*;
use cantrip_mechanics::{copy, effects, targets, tokens};
use cantrip_oracle as oracle;

/// "Target Elemental you control" (CR 115.1, 205.3): a permanent its controller controls that is an Elemental, the creature type
/// read through the words of this card's text (CR 612).
fn elemental_you_control(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if targets::permanent(q, id)
        && q.controller(id) == asker.controller
        && q.chars(id).has_subtype(q.words(asker.source).subtype(Subtype::Elemental)))
}

/// "Draw two cards" for the target player (see `effects::target_player_draws`).
fn draw_two(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::target_player_draws(g, item, 0, 2);
    Ok(())
}

/// "Ashling's Command deals 2 damage to each creature target player controls": the creatures that player controls as it
/// resolves, dealt the damage together (CR 120.2, 608.2c); nothing at all if the player is no longer a legal target
/// (CR 608.2b).
fn two_damage_to_each_creature(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(player) = targets::player(g, item, 0) else { return Ok(()) };
    let hits: Vec<(ObjectId, Target, u16)> = g
        .permanents_of(player)
        .into_iter()
        .filter(|&id| g.chars(id).is(CardType::Creature))
        .map(|id| (item.object, Target::Object(id), 2))
        .collect();
    g.damage_together(&hits);
    Ok(())
}

/// "Target player creates two Treasure tokens" (CR 111.10a): both tokens at once, under that player's control.
fn two_treasures(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(player) = targets::player(g, item, 0) else { return Ok(()) };
    let treasure = g.card_id(&tokens::TREASURE);
    g.create_tokens(treasure, player, 2, false)?;
    Ok(())
}

static MODAL: ModalDef = modes(&[
    mode(copy::create_token_copy_of_target, "Create a token that's a copy of target Elemental you control.")
        .targets(&[target(elemental_you_control)]),
    mode(draw_two, "Target player draws two cards.").targets(&[targets::PLAYER]),
    mode(two_damage_to_each_creature, "Ashling's Command deals 2 damage to each creature target player controls.")
        .targets(&[targets::PLAYER]),
    mode(two_treasures, "Target player creates two Treasure tokens.").targets(&[targets::PLAYER]),
])
.choose(2, 2);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::ASHLINGS_COMMAND };

pub static EXTRA: &[&CardDef] = &[&tokens::TREASURE];
