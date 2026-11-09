use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Destroy all nonblack creatures. Hellfire deals X plus 3 damage to you, where X is the number of creatures that died this way." (CR 608.2, 120.3, 701.8a).
fn destroy_all_nonblack_creatures_hellfire(
    g: &mut dyn Game,
    item: &StackItem,
) -> Result<(), Suspend> {
    let nonblack_creatures = {
        let q: &dyn Query = g;
        q.state()
            .battlefield()
            .iter()
            .copied()
            .filter(|&id| {
                !(q.chars(id).colors.has(q.words(item.object).color(Color::Black)))
                    && q.chars(id).is(CardType::Creature)
            })
            .collect::<Vec<ObjectId>>()
    };
    let destroyed = g.destroy_all(&nonblack_creatures);
    g.end_sentence();
    // A creature died if its own move put it into a graveyard: a merged creature dies once, however many cards it became
    // (CR 730.2, 730.3c).
    let died = destroyed
        .permanents()
        .iter()
        .filter(|&&old| g.obj(old).became.is_some_and(|new| g.obj(new).zone.is(ZoneKind::Graveyard)))
        .count() as u32;
    g.damage_together(&[(item.object, Target::Player(item.controller), died + 3)]);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], destroy_all_nonblack_creatures_hellfire),
    ..oracle::HELLFIRE
};
