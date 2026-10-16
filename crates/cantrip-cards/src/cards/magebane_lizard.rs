use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Whenever a player casts a noncreature spell" (CR 603.2): any player's, this creature's controller's included. The ability is
/// about the player who cast it.
fn player_casts_noncreature_spell(q: &dyn Query, _: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::SpellCast { spell, player, .. } if !q.chars(*spell).is(CardType::Creature) => Fired::about(Target::Player(*player)),
        _ => None,
    }
}

/// "This creature deals damage to that player equal to the number of noncreature spells they've cast this turn": counted as the
/// ability resolves (CR 608.2h), so a noncreature spell cast in response counts too; dealt by this creature as it last was
/// if it has left (CR 113.7a). The spell that triggered it is cast, so the count is at least one.
fn deal_damage(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Target::Player(player) = item.subject else { return Ok(()) };
    let spells = g.history().casts_by(player).filter(|c| !c.is(CardType::Creature)).count();
    g.damage(item.object, Target::Player(player), spells.min(u16::MAX as usize) as u16);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when(EventMask::SPELL_CAST, player_casts_noncreature_spell),
        deal_damage,
        "Whenever a player casts a noncreature spell, this creature deals damage to that player equal to the number of noncreature spells they've cast this turn.",
    )],
    ..oracle::MAGEBANE_LIZARD
};
