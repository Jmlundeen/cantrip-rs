use cantrip_api::*;
use cantrip_mechanics::{effects, targets, triggers};
use cantrip_oracle as oracle;

/// "When this creature enters, exile target spell with mana value 4 or less." The spell leaves the stack without being
/// countered (CR 701.6a), so one that can't be countered is exiled too (its ruling), and
/// is linked to this ability for the leaves-the-battlefield ability to find (CR 607.2a). An exiled copy of a spell
/// ceases to exist (CR 707.10a), leaving nothing to cast. Nothing happens if the spell is no longer a legal target (CR 608.2b).
fn exile_spell(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(spell) = targets::object(g, item, 0) else { return Ok(()) };
    g.remove_spell(spell, CounterTo::Exile);
    if let Some(exiled) = g.obj(spell).became {
        g.link_exiled(&[exiled], g.linked(item));
    }
    Ok(())
}

/// "When this creature leaves the battlefield, the exiled card's owner may cast that card without paying its mana cost."
/// The card exiled with this creature's enter ability (CR 607.2a), still in exile; with nothing exiled (the enter ability
/// has not resolved yet, or its spell was a copy) it does nothing (its ruling). Its owner casts it as this ability
/// resolves or not at all (CR 608.2g, the ruling), its timing waived but not a restriction such as "cast only during
/// combat", choosing its targets and paying its additional costs themselves, X 0 (CR 107.3b, 118.9). Should the enter
/// ability have exiled more than one card (Panharmonicon), each owner in turn order casts any of theirs, in the order they
/// choose (CR 101.4, 101.4c).
fn owner_may_cast(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let exiled: Vec<(PlayerId, ObjectId)> = g.exiled_with(g.linked(item)).into_iter().map(|card| (g.obj(card).owner, card)).collect();
    let players: Vec<PlayerId> = g.state().apnap().collect();
    for player in players {
        let theirs: Vec<ObjectId> = exiled.iter().filter(|&&(owner, _)| owner == player).map(|&(_, card)| card).collect();
        match theirs[..] {
            [] => {}
            [card] => {
                effects::may_cast_without_paying(g, player, card)?;
            }
            _ => cantrip_mechanics::choose::cast_any_number_free(g, player, item.object, &theirs)?,
        }
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(triggers::THIS_ENTERS, exile_spell, "When this creature enters, exile target spell with mana value 4 or less.")
            .targets(&[target(targets::spell_with_mana_value_at_most::<4>)]),
        triggered(
            triggers::THIS_LEAVES,
            owner_may_cast,
            "When this creature leaves the battlefield, the exiled card's owner may cast that card without paying its mana cost.",
        ),
    ],
    ..oracle::SPELL_QUELLER
};
